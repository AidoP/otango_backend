use yew::prelude::*;
use common::data;

#[derive(Properties, PartialEq, Eq)]
pub struct Word {
    pub word: String
}
#[derive(Properties, PartialEq, Eq)]
pub struct Kanji {
    pub kanji: String
}
#[derive(Properties)]
pub struct Tags {
    pub tags: Vec<data::Tag>
}
impl PartialEq for Tags {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}