// use crate::pages::Page;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct App {
    pub active: bool,
    // pub current_page: Page,
    pub tab_counter: u8,
    pub search: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active: true,
            // current_page: Page::Home,
            tab_counter: 0,
            search: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
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
        self.tab_counter += 1;
    }

    pub fn decr_tab(&mut self) {
        self.tab_counter -= 1;
    }
}
