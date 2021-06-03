mod deadfish;
mod lang_trait;
mod s10k;

use lazy_static::lazy_static;
use seed::*;
use std::collections::HashMap;

use deadfish::Deadfish;
use lang_trait::Language;
use s10k::S10K;

lazy_static! {
    static ref HASHMAP: HashMap<&'static str, fn() -> Box<dyn Language>> = {
        let mut m = HashMap::new();
        m.insert(
            "S10K",
            (|| Box::new(S10K::new())) as fn() -> Box<dyn Language>,
        );
        m.insert(
            "Deadfish",
            (|| Box::new(Deadfish::new())) as fn() -> Box<dyn Language>,
        );
        m
    };
    pub static ref LANGS: Vec<&'static str> = {
        let mut v = HASHMAP.keys().cloned().collect::<Vec<_>>();
        v.sort_unstable();
        v
    };
}

pub struct LangContext {
    last_steps: usize,
    lang: Box<dyn Language>,
}

impl LangContext {
    pub fn init(lang: &str, code: &str, input: &str, args: &str) -> Self {
        log!("Lang received:", lang);
        let mut ctx = Self {
            last_steps: 1,
            lang: (HASHMAP.get(lang).unwrap())(),
        };
        ctx.lang.init(code, input, args);
        ctx
    }
    pub fn step(&mut self) -> (String, String, bool) {
        self.lang.step()
    }
    pub fn step_many(&mut self, steps: usize) -> (String, String, bool) {
        self.last_steps = steps;
        let mut is_running = true;
        let mut stdout = String::new();
        let mut stderr = String::new();
        for _ in 0..steps {
            let (next_stdout, next_stderr, next_running) = self.step();
            stdout += &next_stdout;
            stderr += &next_stderr;
            is_running = next_running;
            if !is_running {
                break;
            }
        }
        (stdout, stderr, is_running)
    }
    pub fn step_adaptive(&mut self, delta: f64) -> (String, String, bool) {
        let steps = if delta < 50.0 {
            self.last_steps * 2
        } else {
            (self.last_steps * 2 / 3).max(1)
        };
        log!(delta, steps);
        self.step_many(steps)
    }
}
