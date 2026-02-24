// src/domain.rs

// dependencies
use serde::Deserialize;
use yew::AttrValue;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Quote {
    pub text: AttrValue,
    pub author: AttrValue,
    pub tag: AttrValue,
}
