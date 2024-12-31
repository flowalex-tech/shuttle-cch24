use std::sync::LazyLock;

use actix_web::{get, post, web, HttpResponse};
use rand::{Rng, SeedableRng};
use shuttle_runtime::tokio::sync::Mutex;

static STATE: LazyLock<web::Data<State>> = LazyLock::new(Default::default);

#[derive(Default, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Tile {
    #[default]
    Empty,
    Cookie,
    Milk,
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Empty => '⬛',
            Tile::Cookie => '🍪',
            Tile::Milk => '🥛',
        };

        write!(f, "{c}")
    }
}

#[derive(Default)]
struct State {
    board: Mutex<Board>,
    random_board: Mutex<RandomBoard>,
}

#[derive(Default)]
struct Board {
    inner: [[Tile; 4]; 4],
    won: Option<Tile>,
}

impl std::ops::Deref for Board {
    type Target = [[Tile; 4]; 4];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for row in self.iter() {
            s.push('⬜');

            for tile in row.iter() {
                s.push_str(&tile.to_string());
            }

            s.push_str("⬜\n");
        }
        s.push_str("⬜⬜⬜⬜⬜⬜\n");

        if let Some(tile) = self.won {
            if tile != Tile::Empty {
                s.push_str(&format!("{tile} wins!\n"));
            } else {
                s.push_str("No winner.\n");
            }
        }

        write!(f, "{s}")
    }
}

struct RandomBoard {
    board: Board,
    rng: rand::rngs::StdRng,
}

impl Default for RandomBoard {
    fn default() -> Self {
        let board = Board::default();
        let rng = rand::rngs::StdRng::seed_from_u64(2024);

        Self { board, rng }
    }
}

impl RandomBoard {
    fn gen(&mut self) {
        let board = &mut self.board;
        board.won = Some(Tile::Empty);

        for row in board.iter_mut() {
            for tile in row.iter_mut() {
                *tile = if self.rng.gen::<bool>() {
                    Tile::Cookie
                } else {
                    Tile::Milk
                };
            }
        }

        // horizontal
        for y in 0..4 {
            if board[y].iter().all(|&t| t == Tile::Cookie) {
                board.won = Some(Tile::Cookie);
                return;
            } else if board[y].iter().all(|&t| t == Tile::Milk) {
                board.won = Some(Tile::Milk);
                return;
            }
        }

        // vertical
        for x in 0..4 {
            if (0..board[0].len()).all(|y| board[y][x] == Tile::Cookie) {
                board.won = Some(Tile::Cookie);
                return;
            } else if (0..board[0].len()).all(|y| board[y][x] == Tile::Milk) {
                board.won = Some(Tile::Milk);
                return;
            }
        }

        // tl -> br
        if (0..board.len()).all(|i| board[i][i] == Tile::Cookie) {
            board.won = Some(Tile::Cookie);
            return;
        } else if (0..board.len()).all(|i| board[i][i] == Tile::Milk) {
            board.won = Some(Tile::Milk);
            return;
        }

        // br -> tl
        if (0..board.len()).all(|i| board[board.len() - i - 1][i] == Tile::Cookie) {
            board.won = Some(Tile::Cookie);
        } else if (0..board.len()).all(|i| board[board.len() - i - 1][i] == Tile::Milk) {
            board.won = Some(Tile::Milk);
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    let state = STATE.clone();

    cfg.service(get_board)
        .service(post_reset)
        .service(post_place)
        .service(get_random_board)
        .app_data(state);
}

#[get("/12/board")]
async fn get_board(state: web::Data<State>) -> HttpResponse {
    let board = state.board.lock().await;

    HttpResponse::Ok().body(board.to_string())
}

#[post("/12/reset")]
async fn post_reset(state: web::Data<State>) -> HttpResponse {
    let mut board = state.board.lock().await;
    *board = Default::default();
    let mut random_board = state.random_board.lock().await;
    *random_board = Default::default();

    HttpResponse::Ok().body(board.to_string())
}

#[post("/12/place/{team}/{column}")]
async fn post_place(path: web::Path<(Tile, usize)>, state: web::Data<State>) -> HttpResponse {
    let (team, column) = path.into_inner();

    if !(1..=4).contains(&column) {
        return HttpResponse::BadRequest().finish();
    }
    let column = column - 1;

    let mut board = state.board.lock().await;

    if board.won.is_some() {
        return HttpResponse::ServiceUnavailable().body(board.to_string());
    }

    let Some(y) = board
        .iter()
        .rev()
        .position(|row| row[column] == Tile::Empty)
    else {
        return HttpResponse::ServiceUnavailable().body(board.to_string());
    };
    let y = board.len() - y - 1;

    board[y][column] = team;

    // horizontal
    if board[y].iter().all(|&t| t == team) {
        board.won = Some(team);
    }

    // vertical
    if (0..board[0].len()).all(|y| board[y][column] == team) {
        board.won = Some(team);
    }

    // tl -> br
    if (0..board.len()).all(|i| board[i][i] == team) {
        board.won = Some(team);
    }

    // br -> tl
    if (0..board.len()).all(|i| board[board.len() - i - 1][i] == team) {
        board.won = Some(team);
    }

    // no winner
    if board.iter().all(|r| r.iter().all(|&t| t != Tile::Empty)) {
        board.won = Some(Tile::Empty);
    }

    HttpResponse::Ok().body(board.to_string())
}

#[get("/12/random-board")]
async fn get_random_board(state: web::Data<State>) -> HttpResponse {
    let mut random_board = state.random_board.lock().await;
    random_board.gen();

    HttpResponse::Ok().body(random_board.board.to_string())
}