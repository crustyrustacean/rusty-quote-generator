// src/view/home_view.rs

// dependencies
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HomeViewProps {
    pub children: Children,
}

#[component]
pub fn HomeView(props: &HomeViewProps) -> Html {
    html! {
        <div>
            { props.children.clone() }
        </div>
    }
}
