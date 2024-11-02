#![allow(clippy::needless_range_loop)]
#![allow(dead_code, unused_mut, unused_variables)]
use core::fmt;
use std::{cmp::Ordering, collections::BinaryHeap, env, time::Instant};

use rand::{prelude::*, Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Coord {
    y: i32,
    x: i32,
}

impl Coord {
    fn new(y: i32, x: i32) -> Self {
        Self { y, x }
    }
}

const H: usize = 30;
const W: usize = 30;
const END_TURN: usize = 100;
const NUM_GAME: usize = 100;

struct TimeKeeper {
    start_time: std::time::Instant,
    time_threshold: u128,
}

impl TimeKeeper {
    fn new(time_threshold: u128) -> Self {
        Self {
            start_time: Instant::now(),
            time_threshold,
        }
    }

    fn is_over(&self) -> bool {
        let elapsed_msec = self.start_time.elapsed().as_millis();
        elapsed_msec >= self.time_threshold
    }
}

type State = MazeState;

#[derive(Clone, Eq)]
struct MazeState {
    points: Vec<Vec<usize>>,
    turn: usize,
    character: Coord,
    game_score: usize,
    evaluated_score: usize,
    dx: [i32; 4],
    dy: [i32; 4],
    first_action: usize,
}

impl MazeState {
    fn new(seed: u64) -> Self {
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        let character = Coord {
            y: rng.gen::<i32>().rem_euclid(H as i32),
            x: rng.gen::<i32>().rem_euclid(W as i32),
        };

        let mut points: Vec<Vec<usize>> = vec![vec![0; W]; H];
        for y in 0..H {
            for x in 0..W {
                if y as i32 == character.y && x as i32 == character.x {
                    continue;
                }
                points[y][x] = rng.next_u64() as usize % 10;
            }
        }
        Self {
            points,
            turn: 0,
            character,
            game_score: 0,
            evaluated_score: 0,
            // 0: 右, 1: 左, 2: 下, 3:上
            dx: [1, -1, 0, 0],
            dy: [0, 0, 1, -1],
            first_action: 0,
        }
    }

    /// ゲームの終了判定
    fn is_done(&self) -> bool {
        self.turn == END_TURN
    }

    /// 指定したactionでゲームを１ターン進める
    /// 0: 右, 1: 左, 2: 下, 3:上
    fn advance(&mut self, action: usize) {
        self.character.x += self.dx[action];
        self.character.y += self.dy[action];
        let point = &mut self.points[self.character.y as usize][self.character.x as usize];
        if *point > 0 {
            self.game_score += *point;
            *point = 0;
        }
        self.turn += 1;
    }

    /// プレイヤーが可能な行動を全て取得する
    fn legal_actions(&self) -> Vec<usize> {
        let mut legal_actions = vec![];
        for action in 0..4 {
            let ty = self.character.y + self.dy[action];
            let tx = self.character.x + self.dx[action];
            if 0 <= ty && ty < H as i32 && 0 <= tx && tx < W as i32 {
                legal_actions.push(action);
            }
        }
        legal_actions
    }

    fn evaluate_score(&mut self) {
        self.evaluated_score = self.game_score
    }

    fn greedy_action(&self) -> usize {
        let legal_actions = self.legal_actions();
        assert!(!legal_actions.is_empty());
        let mut best_action = None;
        let mut highest = None;
        for action in legal_actions {
            let next_y = self.character.y + self.dy[action];
            let next_x = self.character.x + self.dx[action];
            assert!(0 <= next_y && next_y < H as i32);
            assert!(0 <= next_x && next_x < W as i32);
            let next_score = self.points[next_y as usize][next_x as usize];
            if highest.is_none() || next_score > highest.unwrap() {
                highest = Some(next_score);
                best_action = Some(action);
            }
        }
        assert!(best_action.is_some());
        best_action.unwrap()
    }
}

impl Ord for MazeState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.evaluated_score.cmp(&other.evaluated_score)
    }
}

impl PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MazeState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score == other.evaluated_score
    }
}

impl fmt::Display for MazeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();
        buf.push_str(&format!("turn:\t{}\n", self.turn));
        buf.push_str(&format!("score:\t{}\n", self.game_score));
        for y in 0..H {
            for x in 0..W {
                if self.character.y == y as i32 && self.character.x == x as i32 {
                    buf.push('@');
                } else if self.points[y][x] > 0 {
                    buf.push(char::from_digit(self.points[y][x] as u32, 10).unwrap());
                } else {
                    buf.push('.');
                }
            }
            buf.push('\n');
        }
        write!(f, "{}", buf)
    }
}

fn random_action(state: &State, rng: &mut ChaCha12Rng) -> usize {
    let legal_actions = state.legal_actions();
    legal_actions[rng.gen::<usize>() % legal_actions.len()]
}

fn greedy_action(state: &State) -> usize {
    let legal_actions = state.legal_actions();
    assert!(!legal_actions.is_empty());
    let mut best_action = None;
    let mut highest = None;
    for action in legal_actions {
        let mut next_state = state.clone();
        next_state.advance(action);
        next_state.evaluate_score();
        if highest.is_none() || highest.unwrap() < next_state.evaluated_score {
            highest = Some(next_state.evaluated_score);
            best_action = Some(action);
        }
    }
    assert!(best_action.is_some());
    best_action.unwrap()
}

fn beam_search_action(state: &State, beam_width: usize, beam_depth: usize) -> usize {
    let mut now_beam = BinaryHeap::new();
    let mut best_state: Option<State> = None;

    now_beam.push(state.clone());

    for t in 0..beam_depth {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legal_actions();
            for action in legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(action);
                next_state.evaluate_score();
                if t == 0 {
                    next_state.first_action = action;
                }
                next_beam.push(next_state);
            }
        }
        now_beam = next_beam;
        assert!(!now_beam.is_empty());
        best_state = Some(now_beam.peek().unwrap().clone());
        if best_state.clone().unwrap().is_done() {
            break;
        }
    }
    assert!(best_state.is_some());

    best_state.unwrap().first_action
}

fn beam_search_action_with_time_threshold(
    state: &State,
    beam_width: usize,
    time_threshold: u128,
) -> usize {
    let mut now_beam = BinaryHeap::new();
    let mut best_state: Option<State> = None;
    let time_keeper = TimeKeeper::new(time_threshold);

    now_beam.push(state.clone());

    for t in 0.. {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            #[cfg(debug_assertions)]
            {
                // eprintln!(
                //     "elapsed time: {}",
                //     time_keeper.start_time.elapsed().as_micros()
                // );
            }
            if time_keeper.is_over() {
                return best_state.unwrap().first_action;
            }
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legal_actions();
            for action in legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(action);
                next_state.evaluate_score();
                if t == 0 {
                    next_state.first_action = action;
                }
                next_beam.push(next_state);
            }
        }
        now_beam = next_beam;
        assert!(!now_beam.is_empty());
        best_state = Some(now_beam.peek().unwrap().clone());
        if best_state.clone().unwrap().is_done() {
            break;
        }
    }
    assert!(best_state.is_some());

    best_state.unwrap().first_action
}

fn chokudai_search_action(
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    beam_num: usize,
) -> usize {
    let mut beams = vec![BinaryHeap::<State>::new(); beam_depth + 1];
    beams[0].push(state.clone());

    for _ in 0..beam_num {
        for t in 0..beam_depth {
            let (first, second) = beams.split_at_mut(t + 1);
            let now_beam = &mut first[t];
            let next_beam = &mut second[0];
            for i in 0..beam_width {
                if now_beam.is_empty() {
                    break;
                }
                let now_state = now_beam.peek().unwrap().clone();
                if now_state.is_done() {
                    break;
                }
                now_beam.pop();
                let legal_actions = now_state.legal_actions();
                for action in legal_actions {
                    let mut next_state = now_state.clone();
                    next_state.advance(action);
                    next_state.evaluate_score();
                    if t == 0 {
                        next_state.first_action = action;
                    }
                    #[cfg(debug_assertions)]
                    {
                        eprintln!("{next_state}");
                    }
                    next_beam.push(next_state);
                }
            }
        }
    }

    for t in (0..=beam_depth).rev() {
        if !beams[t].is_empty() {
            return beams[t].peek().unwrap().first_action;
        }
    }

    unreachable!()
}

fn chokudai_search_action_with_time_threshold(
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    time_threshold: u128,
) -> usize {
    let time_keeper = TimeKeeper::new(time_threshold);
    let mut beams = vec![BinaryHeap::<State>::new(); beam_depth + 1];
    beams[0].push(state.clone());

    for _ in 0.. {
        for t in 0..beam_depth {
            let (first, second) = beams.split_at_mut(t + 1);
            let now_beam = &mut first[t];
            let next_beam = &mut second[0];
            for i in 0..beam_width {
                if now_beam.is_empty() {
                    break;
                }
                let now_state = now_beam.peek().unwrap().clone();
                if now_state.is_done() {
                    break;
                }
                now_beam.pop();
                let legal_actions = now_state.legal_actions();
                for action in legal_actions {
                    let mut next_state = now_state.clone();
                    next_state.advance(action);
                    next_state.evaluate_score();
                    if t == 0 {
                        next_state.first_action = action;
                    }
                    #[cfg(debug_assertions)]
                    {
                        // eprintln!("{next_state}");
                    }
                    next_beam.push(next_state);
                }
            }
        }
        if time_keeper.is_over() {
            break;
        }
    }

    for t in (0..=beam_depth).rev() {
        if !beams[t].is_empty() {
            return beams[t].peek().unwrap().first_action;
        }
    }

    unreachable!()
}
fn play_game(seed: u64) {
    let mut state = State::new(seed);
    println!("{}", state);
    while !state.is_done() {
        state.advance(chokudai_search_action_with_time_threshold(
            &state, 1, END_TURN, 1,
        ));
        #[cfg(debug_assertions)]
        {
            println!("action determined.");
            println!("NEXT STATE:");
            println!("{}", state);
        }
    }
}

fn test_ai_score(num: usize) {
    let mut rng = ChaCha12Rng::seed_from_u64(0);
    let mut score_mean = 0.;

    for seed in 0..num {
        let mut state = State::new(seed as u64);
        while !state.is_done() {
            // state.advance(chokudai_search_action_with_time_threshold(
            //     &state, 2, END_TURN, 10,
            // ));
            state.advance(beam_search_action_with_time_threshold(&state, 5, 10));
        }
        score_mean += state.game_score as f64;
    }

    score_mean /= num as f64;
    println!("score_mean: {score_mean}")
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let seed = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        0
    };
    // play_game(seed)
    test_ai_score(NUM_GAME);
}
