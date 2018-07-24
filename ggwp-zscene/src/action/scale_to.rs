use ggez::graphics::Point2;
use ggez::timer;
use std::time::Duration;
use {Action, Sprite};

#[derive(Debug)]
pub struct ScaleTo {
    sprite: Sprite,
    from: Point2,
    to: Point2,
    duration: Duration,
    progress: Duration,
}

impl ScaleTo {
    pub fn new(sprite: &Sprite, to: Point2, duration: Duration) -> Self {
        Self {
            sprite: sprite.clone(),
            from: sprite.scale(),
            to,
            duration,
            progress: Duration::new(0, 0),
        }
    }
}

impl Action for ScaleTo {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn begin(&mut self) {
        self.from = self.sprite.scale();
    }

    fn end(&mut self) {
        self.sprite.set_scale(self.to);
    }

    fn update(&mut self, mut dtime: Duration) {
        if dtime + self.progress > self.duration {
            dtime = self.duration - self.progress;
        }
        let progress_f = timer::duration_to_f64(self.progress) as f32;
        let duration_f = timer::duration_to_f64(self.duration) as f32;
        let k = progress_f / duration_f;
        let diff = self.to - self.from;
        let scale = self.from + diff * k;
        self.sprite.set_scale(scale);
        self.progress += dtime;
        println!("ScaleTo::update: {:?}", self.sprite);
    }

    fn is_finished(&self) -> bool {
        self.progress >= self.duration
    }
}
