use log::info;
use std::{fmt::Debug, time::Duration};

use ggez::{
    self,
    nalgebra::Point2,
    graphics,
    Context,
};

use crate::ZResult;

mod battle;
mod campaign;
mod main_menu;
mod strategy_map;

pub use self::{
    battle::Battle, campaign::Campaign, main_menu::MainMenu, strategy_map::StrategyMap,
};

#[derive(Debug)]
pub enum Transition {
    None,
    Push(Box<dyn Screen>),
    Pop,
}

pub trait Screen: Debug {
    fn update(&mut self, context: &mut Context, dtime: Duration) -> ZResult<Transition>;
    fn draw(&self, context: &mut Context) -> ZResult;
    fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult<Transition>;
    fn resize(&mut self, aspect_ratio: f32);
}

const ERR_MSG: &str = "Screen stack is empty";

pub struct Screens {
    screens: Vec<Box<dyn Screen>>,
}

impl Screens {
    pub fn new(start_screen: Box<dyn Screen>) -> Self {
        Self {
            screens: vec![start_screen],
        }
    }

    pub fn update(&mut self, context: &mut Context) -> ZResult {
        let dtime = ggez::timer::delta(context);
        let command = self.screen_mut().update(context, dtime)?;
        self.handle_command(context, command)
    }

    pub fn draw(&self, context: &mut Context) -> ZResult {
        let bg_color = [0.9, 0.9, 0.8, 1.0].into();
        graphics::clear(context, bg_color);
        self.screen().draw(context)?;
        graphics::present(context)
    }

    pub fn click(&mut self, context: &mut Context, pos: Point2<f32>) -> ZResult {
        let command = self.screen_mut().click(context, pos)?;
        self.handle_command(context, command)
    }

    pub fn resize(&mut self, aspect_ratio: f32) {
        for screen in &mut self.screens {
            screen.resize(aspect_ratio);
        }
    }

    pub fn handle_command(&mut self, context: &mut Context, command: Transition) -> ZResult {
        match command {
            Transition::None => {}
            Transition::Push(screen) => {
                info!("Screens::handle_command: Push");
                self.screens.push(screen);
            }
            Transition::Pop => {
                info!("Screens::handle_command: Pop");
                if self.screens.len() > 1 {
                    self.screens.pop().expect(ERR_MSG);
                } else {
                    ggez::quit(context);
                }
            }
        }
        Ok(())
    }

    /// Returns a mutable reference to the top screen.
    fn screen_mut(&mut self) -> &mut dyn Screen {
        &mut **self.screens.last_mut().expect(ERR_MSG)
    }

    /// Returns a reference to the top screen.
    fn screen(&self) -> &dyn Screen {
        &**self.screens.last().expect(ERR_MSG)
    }
}
