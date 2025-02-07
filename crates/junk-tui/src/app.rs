use crate::pages::Page;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct App {
    pub active: bool,
    pub current_page: Page,
    pub pool: sqlx::PgPool,
    pub search: bool,
    pub tab: u8,
    pub tickers: Vec<Ticker>,
    pub current_
}

#[derive(Debug, sqlx::FromRow)]
pub struct Ticker {
    pub pk: i32,
    pub symbol: String,
    pub title: String,
    pub industry: String,
}

impl App {
    pub async fn new() -> Self {
        let pool = sqlx::PgPool::connect(
            &dotenv::var("FINDUMP_URL").expect("failed to retrieve Env Var FINDUMP_URL"),
        )
        .await
        .expect("could not connect to findump");
        Self {
            active: true,
            current_page: Page::Home,
            tab: 0,
            search: false,
            tickers: sqlx::query_as("SELECT pk, symbol, title, industry FROM stock.symbols")
                .fetch_all(&pool)
                .await
                .expect("failed to fetch stock.symbols from findump"),
            pool,
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.active = false;
    }

    pub fn search_bar(&mut self) {
        match self.search {
            true => self.search = false,
            false => self.search = true,
        }
    }

    pub fn incr_tab(&mut self) {
        self.tab += 1;
    }

    pub fn decr_tab(&mut self) {
        self.tab -= 1;
    }
}
