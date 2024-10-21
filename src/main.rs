#![allow(clippy::needless_range_loop)]
use core::fmt;
use std::env;

use rand::{prelude::*, Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

struct Coord {
    y: i32,
    x: i32,
}

impl Coord {
    fn new(y: i32, x: i32) -> Self {
        Self { y, x }
    }
}

const H: usize = 3;
const W: usize = 4;
const END_TURN: usize = 4;

type State = MazeState;

struct MazeState {
    points: Vec<Vec<usize>>,
    turn: usize,
    character: Coord,
    game_score: usize,
    dx: [i32; 4],
    dy: [i32; 4],
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
            // 0: 右, 1: 左, 2: 下, 3:上
            dx: [1, -1, 0, 0],
            dy: [0, 0, 1, -1],
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

fn play_game(seed: u64) {
    let mut state = State::new(seed);
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    println!("{}", state);
    while !state.is_done() {
        state.advance(random_action(&state, &mut rng));
        println!("{}", state);
    }
}
fn main() {
    let args: Vec<_> = env::args().collect();
    let seed = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        0
    };
    play_game(seed);
}
