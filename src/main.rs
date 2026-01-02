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

        println!("          Main menu\n");
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
            2 => match load_saved_game() {
                Ok((mut state, saved_board)) => {
                    scoreboard = saved_board;
                    scoreboard.save();
                    run_match(&mut state, &mut scoreboard);
                }
                Err(_) => {
                    println!("\nNo saved game found.");
                    pause();
                }
            },
            3 => view_scoreboard(&scoreboard),
            4 => {
                scoreboard.save();
                println!("\nGoodbye.");
                break;
            }
            _ => {}
        }
    }
}

fn banner() {
    println!("==============================");
    println!("    Rock, Paper, Scissors     ");
    println!("==============================\n");
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
    println!();
}

fn pause() {
    println!("\nPress Enter to continue");
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

fn colorize(s: &str, code: &str) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, s)
}

fn green(s: &str) -> String {
    colorize(s, "32")
}
fn red(s: &str) -> String {
    colorize(s, "31")
}
fn yellow(s: &str) -> String {
    colorize(s, "33")
}
fn cyan(s: &str) -> String {
    colorize(s, "36")
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
            Ruleset::Extended => vec![
                Move::Rock,
                Move::Paper,
                Move::Scissors,
                Move::Lizard,
                Move::Spock,
            ],
        }
    }
}

fn ascii_move(mv: Move) -> &'static str {
    match mv {
        Move::Rock => r#"
    _______
---'   ____)
      (_____)
      (_____)
      (____)
---.__(___)
"#,
        Move::Paper => r#"
     _______
---'   ____)____
          ______)
          _______)
         _______)
---.__________)
"#,
        Move::Scissors => r#"
    _______
---'   ____)____
          ______)
       __________)
      (____)
---.__(___)
"#,
        Move::Lizard => r#"
     __,---._
    /        `.
   |   .-"""-. |
   |  /  _ _  \|
    \ | | | | |
     \| |_| |_||
       \       /
        `-.__.-'
"#,
        Move::Spock => r#"
     🖖
  Live long
  and prosper
"#,
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
    turn: Turn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Turn {
    WaitingP1,
    WaitingP2,
    Reveal,
}

impl MatchState {
    fn new(config: GameConfig) -> Self {
        let turn = match config.mode {
            Mode::SinglePlayer => Turn::WaitingP1,
            Mode::Multiplayer => Turn::WaitingP1,
        };
        Self {
            config,
            round_number: 1,
            p1_round_wins: 0,
            p2_round_wins: 0,
            history: vec![],
            human_recent: vec![],
            turn,
        }
    }

    fn reset_for_rematch(&mut self) {
        self.round_number = 1;
        self.p1_round_wins = 0;
        self.p2_round_wins = 0;
        self.history.clear();
        self.human_recent.clear();
        self.turn = Turn::WaitingP1;
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
        self.players
            .entry(name.to_string())
            .or_insert_with(PlayerStats::default);
    }

    fn add_match_result(
        &mut self,
        p1: &str,
        p2: &str,
        winner: Option<&str>,
        p1_rounds: u32,
        p2_rounds: u32,
    ) {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveData {
    state: MatchState,
    scoreboard: Scoreboard,
}

fn save_game(state: &MatchState, scoreboard: &Scoreboard) {
    let data = SaveData {
        state: state.clone(),
        scoreboard: scoreboard.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(SAVE_FILE, json);
    }
}

fn load_saved_game() -> Result<(MatchState, Scoreboard), ()> {
    let data = fs::read_to_string(SAVE_FILE).map_err(|_| ())?;
    let sd: SaveData = serde_json::from_str(&data).map_err(|_| ())?;
    Ok((sd.state, sd.scoreboard))
}

fn clear_saved_game() {
    let _ = fs::remove_file(SAVE_FILE);
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
        println!("1) Sort by matches won");
        println!("2) Sort by win rate");
        println!("3) Sort by rounds won");
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
            2 => rows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap()),
            3 => rows.sort_by(|a, b| b.1.rounds_won.cmp(&a.1.rounds_won)),
            _ => {}
        }

        clear_screen();
        banner();
        println!(
            "{:<20} {:>6} {:>6} {:>8} {:>10}",
            "Player", "MP", "MW", "RW", "    Win Rate"
        );
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

    println!("New game setup\n");

    println!("Choose mode:");
    println!("1) Single-player");
    println!("2) Multiplayer");
    let mode = match read_menu_choice(1, 2) {
        1 => Mode::SinglePlayer,
        _ => Mode::Multiplayer,
    };

    let player1 = loop {
        let s = read_line("\nPlayer 1 name: ");
        if !s.is_empty() {
            break s;
        }
        println!("Name can't be empty.");
    };

    let (player2, difficulty) = match mode {
        Mode::SinglePlayer => {
            println!("\nChoose difficulty:");
            println!("1) Easy");
            println!("2) Normal");
            println!("3) Hard");
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
    println!("1) Classic");
    println!("2) Extended");
    let ruleset = match read_menu_choice(1, 2) {
        1 => Ruleset::Classic,
        _ => Ruleset::Extended,
    };

    println!("\nChoose match format:");
    println!("1) Single round");
    println!("2) Best of N");
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
                println!("Invalid.");
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
                println!("Invalid.");
            };
            MatchFormat::FirstToK(k)
        }
    };

    GameConfig {
        player1,
        player2,
        mode,
        ruleset,
        format,
        difficulty,
    }
}

fn run_match(state: &mut MatchState, scoreboard: &mut Scoreboard) {
    scoreboard.ensure_player(&state.config.player1);
    scoreboard.ensure_player(&state.config.player2);

    let mut pending_p1: Option<Move> = None;
    let mut pending_p2: Option<Move> = None;

    loop {
        clear_screen();
        banner();
        print_match_header(state);

        println!("Quick actions:");
        println!("1) Continue");
        println!("2) Save now (return to main menu)");
        println!("3) Return to main menu without saving");
        let pre = read_menu_choice(1, 3);
        if pre == 2 {
            save_game(state, scoreboard);
            scoreboard.save();
            return;
        }
        if pre == 3 {
            scoreboard.save();
            return;
        }

        if pending_p1.is_none() && pending_p2.is_none() {
            state.turn = Turn::WaitingP1;
        }

        match state.config.mode {
            Mode::SinglePlayer => {
                let p1 = loop {
                    clear_screen();
                    banner();
                    print_match_header(state);
                    println!("Type 'save' to save now and return to menu.");
                    match read_move_player_or_save(&state.config.player1, state.config.ruleset) {
                        MoveOrSave::Save => {
                            save_game(state, scoreboard);
                            scoreboard.save();
                            return;
                        }
                        MoveOrSave::Move(mv) => break mv,
                    }
                };

                let p2 = ai_move(state, p1);

                let winner = decide_winner(state.config.ruleset, p1, p2);
                apply_round(state, p1, p2, winner);

                clear_screen();
                banner();
                print_round_summary(state, p1, p2, winner);

                if let Some(match_winner) = check_match_winner(state) {
                    pause();
                    handle_match_end(state, scoreboard, match_winner);
                    continue;
                }

                after_round_menu(state, scoreboard);
                state.round_number += 1;
            }
            Mode::Multiplayer => {
                if pending_p1.is_none() {
                    clear_screen();
                    banner();
                    println!("{}'s turn", state.config.player1);
                    println!("Accepted inputs: {}", accepted_inputs_line(state.config.ruleset));
                    println!("Type 'save' to save now and return to menu.");
                    match read_move_hidden_or_save(&state.config.player1, state.config.ruleset) {
                        MoveOrSave::Save => {
                            save_game(state, scoreboard);
                            scoreboard.save();
                            return;
                        }
                        MoveOrSave::Move(mv) => {
                            pending_p1 = Some(mv);
                            state.turn = Turn::WaitingP2;
                            clear_screen();
                            banner();
                            println!("{} locked in.", state.config.player1);
                            println!("Pass to {}.", state.config.player2);
                            pause();
                        }
                    }
                }

                if pending_p2.is_none() {
                    clear_screen();
                    banner();
                    println!("{}'s turn", state.config.player2);
                    println!("Accepted inputs: {}", accepted_inputs_line(state.config.ruleset));
                    println!("Type 'save' to save now and return to menu.");
                    match read_move_hidden_or_save(&state.config.player2, state.config.ruleset) {
                        MoveOrSave::Save => {
                            save_game(state, scoreboard);
                            scoreboard.save();
                            return;
                        }
                        MoveOrSave::Move(mv) => {
                            pending_p2 = Some(mv);
                            state.turn = Turn::Reveal;
                            clear_screen();
                            banner();
                            println!("{} locked in.", state.config.player2);
                            println!("Both moves are locked in. Reveal now.");
                            pause();
                        }
                    }
                }

                if let (Some(p1), Some(p2)) = (pending_p1, pending_p2) {
                    let winner = decide_winner(state.config.ruleset, p1, p2);
                    apply_round(state, p1, p2, winner);

                    clear_screen();
                    banner();
                    print_round_summary(state, p1, p2, winner);

                    pending_p1 = None;
                    pending_p2 = None;

                    if let Some(match_winner) = check_match_winner(state) {
                        pause();
                        handle_match_end(state, scoreboard, match_winner);
                        continue;
                    }

                    after_round_menu(state, scoreboard);
                    state.round_number += 1;
                }
            }
        }
    }
}

fn apply_round(state: &mut MatchState, p1: Move, p2: Move, winner: RoundWinner) {
    match winner {
        RoundWinner::Player1 => state.p1_round_wins += 1,
        RoundWinner::Player2 => state.p2_round_wins += 1,
        RoundWinner::Tie => {}
    }

    state.history.push(RoundRecord {
        round: state.round_number,
        p1_move: p1,
        p2_move: p2,
        winner,
    });
}

fn after_round_menu(state: &MatchState, scoreboard: &mut Scoreboard) {
    loop {
        println!("\nOptions:");
        println!("1) Next round");
        println!("2) View match history");
        println!("3) Save now (return to main menu)");
        println!("4) Return to main menu without saving");

        let opt = read_menu_choice(1, 4);

        if opt == 2 {
            view_match_history(state);
            continue;
        }
        if opt == 3 {
            save_game(state, scoreboard);
            scoreboard.save();
            return;
        }
        if opt == 4 {
            scoreboard.save();
            return;
        }
        break;
    }
}

fn handle_match_end(state: &mut MatchState, scoreboard: &mut Scoreboard, match_winner: RoundWinner) {
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
    clear_saved_game();

    loop {
        println!("\nAfter match:");
        println!("1) Rematch (same settings)");
        println!("2) Change ruleset / format (then rematch)");
        if matches!(state.config.mode, Mode::SinglePlayer) {
            println!("3) Change difficulty (then rematch)");
            println!("4) Return to main menu");
            let c = read_menu_choice(1, 4);
            match c {
                1 => {
                    state.reset_for_rematch();
                    break;
                }
                2 => {
                    change_ruleset_and_format(&mut state.config);
                    state.reset_for_rematch();
                    break;
                }
                3 => {
                    change_difficulty(&mut state.config);
                    state.reset_for_rematch();
                    break;
                }
                4 => return,
                _ => {}
            }
        } else {
            println!("3) Return to main menu");
            let c = read_menu_choice(1, 3);
            match c {
                1 => {
                    state.reset_for_rematch();
                    break;
                }
                2 => {
                    change_ruleset_and_format(&mut state.config);
                    state.reset_for_rematch();
                    break;
                }
                3 => return,
                _ => {}
            }
        }
    }
}

fn change_ruleset_and_format(cfg: &mut GameConfig) {
    clear_screen();
    banner();

    println!("Change ruleset:");
    println!("1) Classic");
    println!("2) Extended");
    cfg.ruleset = match read_menu_choice(1, 2) {
        1 => Ruleset::Classic,
        _ => Ruleset::Extended,
    };

    println!("\nChange match format:");
    println!("1) Single round");
    println!("2) Best of N");
    println!("3) First to K wins");
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
                println!("Invalid.");
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
                println!("Invalid.");
            };
            MatchFormat::FirstToK(k)
        }
    };
}

fn change_difficulty(cfg: &mut GameConfig) {
    clear_screen();
    banner();

    println!("Change difficulty:");
    println!("1) Easy");
    println!("2) Normal");
    println!("3) Hard");
    let diff = match read_menu_choice(1, 3) {
        1 => Difficulty::Easy,
        2 => Difficulty::Normal,
        _ => Difficulty::Hard,
    };
    cfg.difficulty = Some(diff);
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

    println!();

    let score_line = format!(
        "Score: {} {} - {} {}",
        cfg.player1, state.p1_round_wins, state.p2_round_wins, cfg.player2
    );
    println!("{}", cyan(&score_line));

    match cfg.format {
        MatchFormat::SingleRound => {
            println!("This is the only round.\n");
        }
        MatchFormat::BestOfN(n) => {
            let needed = n / 2 + 1;
            let p1_left = needed.saturating_sub(state.p1_round_wins);
            let p2_left = needed.saturating_sub(state.p2_round_wins);
            println!("Wins needed: {}", needed);
            println!(
                "Wins to go: {} needs {}, {} needs {}\n",
                cfg.player1, p1_left, cfg.player2, p2_left
            );
        }
        MatchFormat::FirstToK(k) => {
            let p1_left = k.saturating_sub(state.p1_round_wins);
            let p2_left = k.saturating_sub(state.p2_round_wins);
            println!("Target wins: {}", k);
            println!(
                "Wins to go: {} needs {}, {} needs {}\n",
                cfg.player1, p1_left, cfg.player2, p2_left
            );
        }
    }

    println!("Round: {}\n", state.round_number);
}

fn print_round_summary(state: &MatchState, p1: Move, p2: Move, winner: RoundWinner) {
    let cfg = &state.config;

    println!("Round {}", state.round_number);

    println!("{} chose: {}", cfg.player1, p1.name());
    println!("{}", ascii_move(p1));

    println!("{} chose: {}", cfg.player2, p2.name());
    println!("{}", ascii_move(p2));

    let banner_line = match winner {
        RoundWinner::Tie => yellow("===== TIE ROUND ====="),
        RoundWinner::Player1 => green(&format!("===== {} WINS =====", cfg.player1)),
        RoundWinner::Player2 => green(&format!("===== {} WINS =====", cfg.player2)),
    };
    println!("\n{}", banner_line);

    let p1_result = match winner {
        RoundWinner::Player1 => green("WIN"),
        RoundWinner::Player2 => red("LOSS"),
        RoundWinner::Tie => yellow("TIE"),
    };
    let p2_result = match winner {
        RoundWinner::Player2 => green("WIN"),
        RoundWinner::Player1 => red("LOSS"),
        RoundWinner::Tie => yellow("TIE"),
    };

    println!(
        "{}: {}   |   {}: {}",
        cfg.player1, p1_result, cfg.player2, p2_result
    );

    println!(
        "\nCurrent Score: {} {} - {} {}",
        cfg.player1, state.p1_round_wins, state.p2_round_wins, cfg.player2
    );

    match cfg.format {
        MatchFormat::SingleRound => {}
        MatchFormat::BestOfN(n) => {
            let needed = n / 2 + 1;
            let p1_left = needed.saturating_sub(state.p1_round_wins);
            let p2_left = needed.saturating_sub(state.p2_round_wins);
            println!(
                "Wins to go: {} {}, {} {}",
                cfg.player1, p1_left, cfg.player2, p2_left
            );
        }
        MatchFormat::FirstToK(k) => {
            let p1_left = k.saturating_sub(state.p1_round_wins);
            let p2_left = k.saturating_sub(state.p2_round_wins);
            println!(
                "Wins to go: {} {}, {} {}",
                cfg.player1, p1_left, cfg.player2, p2_left
            );
        }
    }
}

fn view_match_history(state: &MatchState) {
    clear_screen();
    banner();

    if state.history.is_empty() {
        println!("No rounds played yet.");
        pause();
        return;
    }

    for r in &state.history {
        println!(
            "Round {}: {} vs {} => {:?}",
            r.round,
            r.p1_move.name(),
            r.p2_move.name(),
            r.winner
        );
    }

    pause();
}

fn show_victory(state: &MatchState, winner: RoundWinner) {
    let cfg = &state.config;

    println!("Match Complete!\n");

    match winner {
        RoundWinner::Tie => println!("{}", yellow("It ended in a tie.")),
        RoundWinner::Player1 => println!("{}", green(&format!("Winner: {}", cfg.player1))),
        RoundWinner::Player2 => println!("{}", green(&format!("Winner: {}", cfg.player2))),
    }

    let final_score = format!(
        "Final Score: {} {} - {} {}",
        cfg.player1, state.p1_round_wins, state.p2_round_wins, cfg.player2
    );
    println!("\n{}", cyan(&final_score));
}

fn check_match_winner(state: &MatchState) -> Option<RoundWinner> {
    match state.config.format {
        MatchFormat::SingleRound => state.history.last().map(|r| r.winner),
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

enum MoveOrSave {
    Move(Move),
    Save,
}

fn parse_move_or_save(input: &str, ruleset: Ruleset) -> Option<MoveOrSave> {
    let t = input.trim().to_lowercase();
    if t == "save" || t == "sv" {
        return Some(MoveOrSave::Save);
    }
    parse_move(&t, ruleset).map(MoveOrSave::Move)
}

fn read_move_player_or_save(player_name: &str, ruleset: Ruleset) -> MoveOrSave {
    loop {
        println!("Accepted inputs: {}", accepted_inputs_line(ruleset));
        let s = read_line(&format!("{} move: ", player_name));
        if let Some(v) = parse_move_or_save(&s, ruleset) {
            return v;
        }
        println!("Invalid move.");
    }
}

fn read_move_hidden_or_save(_player_name: &str, ruleset: Ruleset) -> MoveOrSave {
    loop {
        let s = read_password().unwrap_or_default();
        if let Some(v) = parse_move_or_save(&s, ruleset) {
            return v;
        }
        println!("Invalid move.");
    }
}

fn parse_move(input: &str, ruleset: Ruleset) -> Option<Move> {
    match input.trim().to_lowercase().as_str() {
        "rock" | "r" => Some(Move::Rock),
        "paper" | "p" => Some(Move::Paper),
        "scissors" | "s" => Some(Move::Scissors),
        "lizard" | "l" if matches!(ruleset, Ruleset::Extended) => Some(Move::Lizard),
        "spock" | "k" if matches!(ruleset, Ruleset::Extended) => Some(Move::Spock),
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
    state.human_recent.push(human_move);
    if state.human_recent.len() > 12 {
        state.human_recent.remove(0);
    }

    let rules = state.config.ruleset;
    let all = Move::all_for_ruleset(rules);
    let diff = state.config.difficulty.unwrap_or(Difficulty::Easy);

    match diff {
        Difficulty::Easy => random_from(&all),
        Difficulty::Normal => {
            let roll: u8 = rand::thread_rng().gen_range(0..100);
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
    let idx = rand::thread_rng().gen_range(0..list.len());
    list[idx]
}

fn most_common(list: &[Move]) -> Option<Move> {
    let mut freq: HashMap<Move, usize> = HashMap::new();
    for &m in list {
        *freq.entry(m).or_insert(0) += 1;
    }
    freq.into_iter().max_by_key(|(_, c)| *c).map(|(m, _)| m)
}

fn best_counter(ruleset: Ruleset, target: Move) -> Move {
    let candidates: Vec<Move> = Move::all_for_ruleset(ruleset)
        .into_iter()
        .filter(|&m| match ruleset {
            Ruleset::Classic => classic_beats(m, target),
            Ruleset::Extended => extended_beats(m, target),
        })
        .collect();

    if candidates.is_empty() {
        target
    } else {
        random_from(&candidates)
    }
}
