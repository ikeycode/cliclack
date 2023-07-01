use std::fmt::Display;
use std::io;

use console::Key;

use crate::{
    prompt::{
        cursor::StringCursor,
        interaction::{Event, PromptInteraction, State},
    },
    theme::{ClackTheme, Theme},
    validate::Validate,
};

type ValidationCallback = Box<dyn Fn(&String) -> Result<(), String>>;

pub struct Password {
    prompt: String,
    input: StringCursor,
    mask: char,
    validate: Option<ValidationCallback>,
}

impl Password {
    pub fn new(prompt: impl Display) -> Self {
        Self {
            prompt: prompt.to_string(),
            input: StringCursor::default(),
            mask: ClackTheme.password_mask(),
            validate: None,
        }
    }

    pub fn mask(mut self, mask: char) -> Self {
        self.mask = mask;
        self
    }

    pub fn validate<V>(mut self, validator: V) -> Self
    where
        V: Validate<String> + 'static,
        V::Err: ToString,
    {
        self.validate = Some(Box::new(move |input: &String| {
            validator.validate(input).map_err(|err| err.to_string())
        }));
        self
    }

    pub fn interact(&mut self) -> io::Result<String> {
        <Self as PromptInteraction<String>>::interact(self)
    }
}

impl PromptInteraction<String> for Password {
    fn input(&mut self) -> Option<&mut StringCursor> {
        Some(&mut self.input)
    }

    fn on(&mut self, event: &Event) -> State {
        let Event::Key(key) = event;

        if *key == Key::Enter {
            if let Some(validator) = &self.validate {
                if let Err(err) = validator(&self.input.to_string()) {
                    return State::Error(err);
                }
            }
            return State::Submit(self.input.to_string());
        }

        State::Active
    }

    fn render(&mut self, state: &State) -> String {
        let mut masked = self.input.clone();
        for chr in masked.iter_mut() {
            *chr = self.mask;
        }

        let line1 = ClackTheme.format_header(&state.into(), &self.prompt);
        let line2 = ClackTheme.format_input(&state.into(), &masked);
        let line3 = ClackTheme.format_footer(&state.into());

        line1 + &line2 + &line3
    }
}