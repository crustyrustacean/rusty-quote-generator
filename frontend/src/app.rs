// frontend/src/app.rs

// dependencies
use crate::components::quote_card::QuoteCard;
use crate::views::HomeView;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    html! {
        <HomeView>
            <QuoteCard />
        </HomeView>
    }
}
