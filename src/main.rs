use rand::Rng;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

const SAVE_FILE: &str = "rps_save.json";
const SCORE_FILE: &str = "rps_scoreboard.json";

fn main() {
    let mut scoreboard = Scoreboard::load();

    loop {
        clear_screen();
        banner();

        println!("Main Menu");
        println!("1) Start a new game");
        println!("2) Continue the saved game");
        println!("3) View the scoreboard");
        println!("4) Exit");

        match read_menu_choice(1, 4) {
            1 => {
                let config = new_game_setup();
                let mut state = MatchState::new(config);
                run_match(&mut state, &mut scoreboard);
            }
            2 => match MatchState::load() {
                Ok(mut state) => run_match(&mut state, &mut scoreboard),
                Err(_) => {
                    println!("\nNo saved game found.");
                    pause();
                }
            },
            3 => view_scoreboard(&scoreboard),
            4 => {
                scoreboard.save();
                println!("\nBye!");
                break;
            }
            _ => {}
        }
    }
}

fn banner() {
    println!("==============================");
    println!("      ROCK - PAPER - SCISSORS ");
    println!("==============================\n");
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
    for _ in 0..1 {
        println!();
    }
}

fn pause() {
    println!("\nPress Enter to continue...");
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
}

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return String::new();
    }
    input.trim().to_string()
}

fn read_menu_choice(min: i32, max: i32) -> i32 {
    loop {
        let s = read_line("\nChoose: ");
        if let Ok(n) = s.parse::<i32>() {
            if n >= min && n <= max {
                return n;
            }
        }
        println!("Invalid choice. Try again.");
    }
}

fn read_yes_no(prompt: &str, default_yes: bool) -> bool {
    loop {
        let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
        let s = read_line(&format!("{} {} ", prompt, hint)).to_lowercase();
        if s.is_empty() {
            return default_yes;
        }
        match s.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please type y or n."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Mode {
    SinglePlayer,
    Multiplayer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Ruleset {
    Classic,
    Extended,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum MatchFormat {
    SingleRound,
    BestOfN(u32),
    FirstToK(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameConfig {
    player1: String,
    player2: String,
    mode: Mode,
    ruleset: Ruleset,
    format: MatchFormat,
    difficulty: Option<Difficulty>,
    use_color: bool,
    show_ascii: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Move {
    Rock,
    Paper,
    Scissors,
    Lizard,
    Spock,
}

impl Move {
    fn name(&self) -> &'static str {
        match self {
            Move::Rock => "Rock",
            Move::Paper => "Paper",
            Move::Scissors => "Scissors",
            Move::Lizard => "Lizard",
            Move::Spock => "Spock",
        }
    }

    fn all_for_ruleset(r: Ruleset) -> Vec<Move> {
        match r {
            Ruleset::Classic => vec![Move::Rock, Move::Paper, Move::Scissors],
            Ruleset::Extended => vec![Move::Rock, Move::Paper, Move::Scissors, Move::Lizard, Move::Spock],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundRecord {
    round: u32,
    p1_move: Move,
    p2_move: Move,
    winner: RoundWinner,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum RoundWinner {
    Player1,
    Player2,
    Tie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchState {
    config: GameConfig,
    round_number: u32,
    p1_round_wins: u32,
    p2_round_wins: u32,
    history: Vec<RoundRecord>,
    human_recent: Vec<Move>,
}

impl MatchState {
    fn new(config: GameConfig) -> Self {
        Self {
            config,
            round_number: 0,
            p1_round_wins: 0,
            p2_round_wins: 0,
            history: vec![],
            human_recent: vec![],
        }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(SAVE_FILE, json);
        }
    }

    fn load() -> Result<Self, ()> {
        let data = fs::read_to_string(SAVE_FILE).map_err(|_| ())?;
        serde_json::from_str(&data).map_err(|_| ())
    }

    fn clear_saved() {
        let _ = fs::remove_file(SAVE_FILE);
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PlayerStats {
    matches_played: u32,
    matches_won: u32,
    rounds_won: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Scoreboard {
    players: HashMap<String, PlayerStats>,
}

impl Scoreboard {
    fn load() -> Self {
        let Ok(data) = fs::read_to_string(SCORE_FILE) else {
            return Scoreboard::default();
        };
        serde_json::from_str(&data).unwrap_or_default()
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(SCORE_FILE, json);
        }
    }

    fn ensure_player(&mut self, name: &str) {
        self.players.entry(name.to_string()).or_insert_with(PlayerStats::default);
    }

    fn add_match_result(&mut self, p1: &str, p2: &str, winner: Option<&str>, p1_rounds: u32, p2_rounds: u32) {
        self.ensure_player(p1);
        self.ensure_player(p2);

        {
            let s1 = self.players.get_mut(p1).unwrap();
            s1.matches_played += 1;
            s1.rounds_won += p1_rounds;
        }
        {
            let s2 = self.players.get_mut(p2).unwrap();
            s2.matches_played += 1;
            s2.rounds_won += p2_rounds;
        }

        if let Some(w) = winner {
            if let Some(sw) = self.players.get_mut(w) {
                sw.matches_won += 1;
            }
        }
    }
}

fn view_scoreboard(scoreboard: &Scoreboard) {
    loop {
        clear_screen();
        banner();

        if scoreboard.players.is_empty() {
            println!("Scoreboard is empty.");
            pause();
            return;
        }

        println!("Scoreboard");
        println!("1) Sort by Matches Won");
        println!("2) Sort by Win Rate");
        println!("3) Sort by Rounds Won");
        println!("4) Back");

        let choice = read_menu_choice(1, 4);
        if choice == 4 {
            return;
        }

        let mut rows: Vec<(String, PlayerStats, f32)> = scoreboard
            .players
            .iter()
            .map(|(name, stats)| {
                let win_rate = if stats.matches_played == 0 {
                    0.0
                } else {
                    stats.matches_won as f32 / stats.matches_played as f32
                };
                (name.clone(), stats.clone(), win_rate)
            })
            .collect();

        match choice {
            1 => rows.sort_by(|a, b| b.1.matches_won.cmp(&a.1.matches_won)),
            2 => rows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)),
            3 => rows.sort_by(|a, b| b.1.rounds_won.cmp(&a.1.rounds_won)),
            _ => {}
        }

        clear_screen();
        banner();
        println!("{:<20} {:>6} {:>6} {:>8} {:>10}", "Player", "MP", "MW", "RW", "WinRate");
        println!("{}", "-".repeat(56));

        for (name, st, wr) in rows {
            println!(
                "{:<20} {:>6} {:>6} {:>8} {:>9.0}%",
                name,
                st.matches_played,
                st.matches_won,
                st.rounds_won,
                wr * 100.0
            );
        }

        pause();
    }
}

fn new_game_setup() -> GameConfig {
    clear_screen();
    banner();

    println!("New Game Setup\n");

    let player1 = loop {
        let s = read_line("Player 1 name: ");
        if !s.is_empty() {
            break s;
        }
        println!("Name can't be empty.");
    };

    println!("\nChoose mode:");
    println!("1) Single Player");
    println!("2) Multiplayer");
    let mode = match read_menu_choice(1, 2) {
        1 => Mode::SinglePlayer,
        _ => Mode::Multiplayer,
    };

    let (player2, difficulty) = match mode {
        Mode::SinglePlayer => {
            println!("\nChoose difficulty:");
            println!("1) Easy (random)");
            println!("2) Normal (semi-random)");
            println!("3) Hard (tracks your habits)");
            let diff = match read_menu_choice(1, 3) {
                1 => Difficulty::Easy,
                2 => Difficulty::Normal,
                _ => Difficulty::Hard,
            };
            ("Computer".to_string(), Some(diff))
        }
        Mode::Multiplayer => {
            let p2 = loop {
                let s = read_line("Player 2 name: ");
                if !s.is_empty() && s != player1 {
                    break s;
                }
                println!("Name can't be empty and must be different from Player 1.");
            };
            (p2, None)
        }
    };

    println!("\nChoose ruleset:");
    println!("1) Classic (Rock, Paper, Scissors)");
    println!("2) Extended (Rock, Paper, Scissors, Lizard, Spock)");
    let ruleset = match read_menu_choice(1, 2) {
        1 => Ruleset::Classic,
        _ => Ruleset::Extended,
    };

    println!("\nChoose match format:");
    println!("1) Single round");
    println!("2) Best of N (odd number like 3, 5, 7)");
    println!("3) First to K wins");
    let format = match read_menu_choice(1, 3) {
        1 => MatchFormat::SingleRound,
        2 => {
            let n = loop {
                let s = read_line("Enter N (odd number >= 1): ");
                if let Ok(v) = s.parse::<u32>() {
                    if v >= 1 && v % 2 == 1 {
                        break v;
                    }
                }
                println!("Invalid. N must be an odd number like 3, 5, 7.");
            };
            MatchFormat::BestOfN(n)
        }
        _ => {
            let k = loop {
                let s = read_line("Enter K (>= 1): ");
                if let Ok(v) = s.parse::<u32>() {
                    if v >= 1 {
                        break v;
                    }
                }
                println!("Invalid. K must be 1 or more.");
            };
            MatchFormat::FirstToK(k)
        }
    };

    let use_color = should_use_color() && read_yes_no("\nUse colors for results?", true);
    let show_ascii = read_yes_no("Show ASCII graphics?", true);

    GameConfig {
        player1,
        player2,
        mode,
        ruleset,
        format,
        difficulty,
        use_color,
        show_ascii,
    }
}

fn should_use_color() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    true
}

fn run_match(state: &mut MatchState, scoreboard: &mut Scoreboard) {
    scoreboard.ensure_player(&state.config.player1);
    scoreboard.ensure_player(&state.config.player2);

    loop {
        clear_screen();
        banner();
        print_match_header(state);

        state.round_number += 1;

        let p1_move = match state.config.mode {
            Mode::SinglePlayer => read_move_player(&state.config.player1, state.config.ruleset),
            Mode::Multiplayer => read_move_hidden(&state.config.player1, state.config.ruleset),
        };

        let p2_move = match state.config.mode {
            Mode::SinglePlayer => ai_move(state, p1_move),
            Mode::Multiplayer => read_move_hidden(&state.config.player2, state.config.ruleset),
        };

        let winner = decide_winner(state.config.ruleset, p1_move, p2_move);

        match winner {
            RoundWinner::Player1 => state.p1_round_wins += 1,
            RoundWinner::Player2 => state.p2_round_wins += 1,
            RoundWinner::Tie => {}
        }

        state.history.push(RoundRecord {
            round: state.round_number,
            p1_move,
            p2_move,
            winner,
        });

        clear_screen();
        banner();
        print_round_summary(state, p1_move, p2_move, winner);

        println!("\nOptions:");
        println!("1) Next round");
        println!("2) View match history");
        println!("3) Save and return to main menu");
        println!("4) Return to main menu without saving");

        let opt = read_menu_choice(1, 4);

        if opt == 2 {
            view_match_history(state);
            continue;
        }
        if opt == 3 {
            state.save();
            scoreboard.save();
            return;
        }
        if opt == 4 {
            scoreboard.save();
            return;
        }

        if let Some(match_winner) = check_match_winner(state) {
            clear_screen();
            banner();
            show_victory(state, match_winner);

            let winner_name = match match_winner {
                RoundWinner::Player1 => Some(state.config.player1.as_str()),
                RoundWinner::Player2 => Some(state.config.player2.as_str()),
                RoundWinner::Tie => None,
            };

            scoreboard.add_match_result(
                &state.config.player1,
                &state.config.player2,
                winner_name,
                state.p1_round_wins,
                state.p2_round_wins,
            );
            scoreboard.save();
            MatchState::clear_saved();

            println!("\nAfter match:");
            println!("1) Rematch with same settings");
            println!("2) Change ruleset or format");
            println!("3) View match history");
            println!("4) Return to main menu");

            let post = read_menu_choice(1, 4);

            match post {
                1 => {
                    let cfg = state.config.clone();
                    *state = MatchState::new(cfg);
                    continue;
                }
                2 => {
                    let mut cfg = state.config.clone();

                    println!("\nChoose ruleset:");
                    println!("1) Classic");
                    println!("2) Extended");
                    cfg.ruleset = match read_menu_choice(1, 2) {
                        1 => Ruleset::Classic,
                        _ => Ruleset::Extended,
                    };

                    println!("\nChoose match format:");
                    println!("1) Single round");
                    println!("2) Best of N (odd)");
                    println!("3) First to K");
                    cfg.format = match read_menu_choice(1, 3) {
                        1 => MatchFormat::SingleRound,
                        2 => {
                            let n = loop {
                                let s = read_line("Enter N (odd number >= 1): ");
                                if let Ok(v) = s.parse::<u32>() {
                                    if v >= 1 && v % 2 == 1 {
                                        break v;
                                    }
                                }
                                println!("Invalid. N must be odd like 3, 5, 7.");
                            };
                            MatchFormat::BestOfN(n)
                        }
                        _ => {
                            let k = loop {
                                let s = read_line("Enter K (>= 1): ");
                                if let Ok(v) = s.parse::<u32>() {
                                    if v >= 1 {
                                        break v;
                                    }
                                }
                                println!("Invalid. K must be 1 or more.");
                            };
                            MatchFormat::FirstToK(k)
                        }
                    };

                    *state = MatchState::new(cfg);
                    continue;
                }
                3 => {
                    view_match_history(state);
                    return;
                }
                _ => return,
            }
        }
    }
}

fn print_match_header(state: &MatchState) {
    let cfg = &state.config;

    println!("Match");
    println!("{} vs {}", cfg.player1, cfg.player2);

    let ruleset = match cfg.ruleset {
        Ruleset::Classic => "Classic",
        Ruleset::Extended => "Extended",
    };
    println!("Ruleset: {}", ruleset);

    let fmt = match cfg.format {
        MatchFormat::SingleRound => "Single round".to_string(),
        MatchFormat::BestOfN(n) => format!("Best of {}", n),
        MatchFormat::FirstToK(k) => format!("First to {} wins", k),
    };
    println!("Format: {}", fmt);

    if let Some(d) = cfg.difficulty {
        let ds = match d {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        };
        println!("Difficulty: {}", ds);
    }

    println!(
        "\nScore: {} {} - {} {}",
        cfg.player1, state.p1_round_wins, state.p2_round_wins, cfg.player2
    );
    println!("Round: {}\n", state.round_number + 1);
}

fn round_banner(w: RoundWinner, p1: &str, p2: &str) -> String {
    match w {
        RoundWinner::Tie => "===== TIE ROUND =====".to_string(),
        RoundWinner::Player1 => format!("===== {} WINS THE ROUND =====", p1),
        RoundWinner::Player2 => format!("===== {} WINS THE ROUND =====", p2),
    }
}

fn move_ascii(m: Move) -> &'static str {
    match m {
        Move::Rock => "    _______\n---'   ____)\n      (_____)\n      (_____)\n      (____)\n---.__(___)\n",
        Move::Paper => "     _______\n---'   ____)____\n          ______)\n          _______)\n         _______)\n---.__________)\n",
        Move::Scissors => "    _______\n---'   ____)____\n          ______)\n       __________)\n      (____)\n---.__(___)\n",
        Move::Lizard => "   (\\_/)\n   ( •_•)\n  / > 🦎\n",
        Move::Spock => "    🖖\n  SPOCK\n",
    }
}

fn print_round_summary(state: &MatchState, p1: Move, p2: Move, winner: RoundWinner) {
    let cfg = &state.config;

    println!("Round {}", state.round_number);
    println!("{} chose: {}", cfg.player1, p1.name());
    println!("{} chose: {}", cfg.player2, p2.name());

    println!("\n{}", round_banner(winner, &cfg.player1, &cfg.player2));

    if cfg.show_ascii {
        println!("\n{}:\n{}", cfg.player1, move_ascii(p1));
        println!("{}:\n{}", cfg.player2, move_ascii(p2));
    }

    let result_text = match winner {
        RoundWinner::Tie => "Result: Tie",
        RoundWinner::Player1 => "Result: Player 1 wins the round",
        RoundWinner::Player2 => "Result: Player 2 wins the round",
    };

    println!("\n{}", colorize(cfg.use_color, winner, result_text));

    println!(
        "\nCurrent Score: {} {} - {} {}",
        cfg.player1, state.p1_round_wins, state.p2_round_wins, cfg.player2
    );

    match cfg.format {
        MatchFormat::BestOfN(n) => {
            let needed = n / 2 + 1;
            println!("First to {} round wins takes the match.", needed);
        }
        MatchFormat::FirstToK(k) => {
            println!("First to {} round wins takes the match.", k);
        }
        MatchFormat::SingleRound => {}
    }
}

fn colorize(use_color: bool, winner: RoundWinner, s: &str) -> String {
    if !use_color {
        return s.to_string();
    }
    match winner {
        RoundWinner::Player1 => format!("\x1b[32m{}\x1b[0m", s),
        RoundWinner::Player2 => format!("\x1b[31m{}\x1b[0m", s),
        RoundWinner::Tie => format!("\x1b[33m{}\x1b[0m", s),
    }
}

fn view_match_history(state: &MatchState) {
    clear_screen();
    banner();

    println!("Match History\n");

    if state.history.is_empty() {
        println!("No rounds played yet.");
        pause();
        return;
    }

    for r in &state.history {
        let w = match r.winner {
            RoundWinner::Tie => "Tie".to_string(),
            RoundWinner::Player1 => format!("Winner: {}", state.config.player1),
            RoundWinner::Player2 => format!("Winner: {}", state.config.player2),
        };

        println!(
            "Round {:>2}: {} = {:<8} | {} = {:<8} | {}",
            r.round,
            state.config.player1,
            r.p1_move.name(),
            state.config.player2,
            r.p2_move.name(),
            w
        );
    }

    pause();
}

fn show_victory(state: &MatchState, winner: RoundWinner) {
    let cfg = &state.config;

    println!("Match Complete!\n");

    match winner {
        RoundWinner::Tie => println!("It ended in a tie."),
        RoundWinner::Player1 => println!("Winner: {}", cfg.player1),
        RoundWinner::Player2 => println!("Winner: {}", cfg.player2),
    }

    println!(
        "\nFinal Score: {} {} - {} {}",
        cfg.player1, state.p1_round_wins, state.p2_round_wins, cfg.player2
    );

    println!("\nShort match summary:");
    let total_rounds = state.history.len();
    let ties = state.history.iter().filter(|r| matches!(r.winner, RoundWinner::Tie)).count();
    println!("Rounds played: {}", total_rounds);
    println!("Ties: {}", ties);
}

fn check_match_winner(state: &MatchState) -> Option<RoundWinner> {
    match state.config.format {
        MatchFormat::SingleRound => {
            if state.round_number >= 1 {
                let last = state.history.last().unwrap();
                Some(last.winner)
            } else {
                None
            }
        }
        MatchFormat::BestOfN(n) => {
            let needed = n / 2 + 1;
            if state.p1_round_wins >= needed {
                Some(RoundWinner::Player1)
            } else if state.p2_round_wins >= needed {
                Some(RoundWinner::Player2)
            } else {
                None
            }
        }
        MatchFormat::FirstToK(k) => {
            if state.p1_round_wins >= k {
                Some(RoundWinner::Player1)
            } else if state.p2_round_wins >= k {
                Some(RoundWinner::Player2)
            } else {
                None
            }
        }
    }
}

fn accepted_inputs_line(ruleset: Ruleset) -> &'static str {
    match ruleset {
        Ruleset::Classic => "rock / paper / scissors  OR  r / p / s",
        Ruleset::Extended => "rock / paper / scissors / lizard / spock  OR  r / p / s / l / k",
    }
}

fn read_move_player(player_name: &str, ruleset: Ruleset) -> Move {
    loop {
        println!("Accepted inputs: {}", accepted_inputs_line(ruleset));
        let s = read_line(&format!("{} move: ", player_name));
        if let Some(mv) = parse_move(&s, ruleset) {
            return mv;
        }
        println!("Invalid move. Try again.\n");
    }
}

fn read_move_hidden(player_name: &str, ruleset: Ruleset) -> Move {
    clear_screen();
    banner();

    println!("{}'s turn (input hidden)", player_name);
    println!("Accepted inputs: {}", accepted_inputs_line(ruleset));

    loop {
        print!("\nType your move (hidden): ");
        let _ = io::stdout().flush();

        let s = read_password().unwrap_or_default();

        if let Some(mv) = parse_move(&s, ruleset) {
            return mv;
        }
        println!("Invalid move. Try again.");
    }
}

fn parse_move(input: &str, ruleset: Ruleset) -> Option<Move> {
    let s = input.trim().to_lowercase();

    match s.as_str() {
        "rock" | "r" => Some(Move::Rock),
        "paper" | "p" => Some(Move::Paper),
        "scissors" | "s" => Some(Move::Scissors),
        "lizard" | "l" => {
            if matches!(ruleset, Ruleset::Extended) {
                Some(Move::Lizard)
            } else {
                None
            }
        }
        "spock" | "k" => {
            if matches!(ruleset, Ruleset::Extended) {
                Some(Move::Spock)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn decide_winner(ruleset: Ruleset, p1: Move, p2: Move) -> RoundWinner {
    if p1 == p2 {
        return RoundWinner::Tie;
    }

    let p1_wins = match ruleset {
        Ruleset::Classic => classic_beats(p1, p2),
        Ruleset::Extended => extended_beats(p1, p2),
    };

    if p1_wins {
        RoundWinner::Player1
    } else {
        RoundWinner::Player2
    }
}

fn classic_beats(a: Move, b: Move) -> bool {
    matches!(
        (a, b),
        (Move::Rock, Move::Scissors) | (Move::Paper, Move::Rock) | (Move::Scissors, Move::Paper)
    )
}

fn extended_beats(a: Move, b: Move) -> bool {
    matches!(
        (a, b),
        (Move::Rock, Move::Scissors)
            | (Move::Rock, Move::Lizard)
            | (Move::Paper, Move::Rock)
            | (Move::Paper, Move::Spock)
            | (Move::Scissors, Move::Paper)
            | (Move::Scissors, Move::Lizard)
            | (Move::Lizard, Move::Spock)
            | (Move::Lizard, Move::Paper)
            | (Move::Spock, Move::Scissors)
            | (Move::Spock, Move::Rock)
    )
}

fn ai_move(state: &mut MatchState, human_move: Move) -> Move {
    if state.config.mode == Mode::SinglePlayer {
        state.human_recent.push(human_move);
        if state.human_recent.len() > 12 {
            state.human_recent.remove(0);
        }
    }

    let rules = state.config.ruleset;
    let all = Move::all_for_ruleset(rules);
    let diff = state.config.difficulty.unwrap_or(Difficulty::Easy);

    match diff {
        Difficulty::Easy => random_from(&all),
        Difficulty::Normal => {
            let mut rng = rand::thread_rng();
            let roll: u8 = rng.gen_range(0..100);
            if roll < 65 {
                random_from(&all)
            } else {
                best_counter(rules, human_move)
            }
        }
        Difficulty::Hard => {
            let predicted = most_common(&state.human_recent).unwrap_or(human_move);
            best_counter(rules, predicted)
        }
    }
}

fn random_from(list: &[Move]) -> Move {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..list.len());
    list[idx]
}

fn most_common(list: &[Move]) -> Option<Move> {
    if list.is_empty() {
        return None;
    }
    let mut freq: HashMap<Move, usize> = HashMap::new();
    for &m in list {
        *freq.entry(m).or_insert(0) += 1;
    }
    freq.into_iter().max_by_key(|(_, c)| *c).map(|(m, _)| m)
}

fn best_counter(ruleset: Ruleset, target: Move) -> Move {
    let candidates = Move::all_for_ruleset(ruleset)
        .into_iter()
        .filter(|&m| {
            let wins = match ruleset {
                Ruleset::Classic => classic_beats(m, target),
                Ruleset::Extended => extended_beats(m, target),
            };
            wins
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        target
    } else {
        random_from(&candidates)
    }
}
