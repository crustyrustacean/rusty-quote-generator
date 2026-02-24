// src/components/quote_card.rs

// dependencies
use crate::domain::Quote;
use gloo_net::http::Request;
use yew::prelude::*;

const API_URL: &str = "https://jacintodesign.github.io/quotes-api/data/quotes.json";

#[function_component(QuoteCard)]
pub fn quote_card() -> Html {
    /* --- STATE --- */
    let quotes = use_state(Vec::new);
    let selected_quote = use_state(|| None::<Quote>);
    let is_loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    /* --- SIDE EFFECTS (Fetching) --- */
    {
        let quotes = quotes.clone();
        let is_loading = is_loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match Request::get(API_URL).send().await {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<Vec<Quote>>().await {
                            Ok(data) => quotes.set(data),
                            Err(_) => error.set(Some("Failed to parse JSON".into())),
                        }
                    }
                    Ok(resp) => error.set(Some(format!("Server Error: {}", resp.status()))),
                    Err(_) => error.set(Some("Network Error: Is the API up?".into())),
                }
                is_loading.set(false);
            });
            || ()
        });
    }

    /* --- EVENT HANDLERS --- */
    let on_new_quote = {
        let quotes = quotes.clone();
        let selected_quote = selected_quote.clone();
        Callback::from(move |_| {
            if !quotes.is_empty() {
                let index = (js_sys::Math::random() * quotes.len() as f64) as usize;
                selected_quote.set(Some(quotes[index].clone()));
            }
        })
    };

    let on_tweet = {
        let selected_quote = selected_quote.clone();
        Callback::from(move |_| {
            // Logic for default values if nothing is selected yet
            let (text, author) = match &*selected_quote {
                Some(q) => (q.text.as_str(), q.author.as_str()),
                None => ("Click 'New Quote' to get started!", "Unknown"),
            };

            let tweet_url = format!(
                "https://twitter.com/intent/tweet?text=\"{}\" - {}",
                text, author
            );
            
            // let _ = sinks the Result<(), JsValue> to satisfy the Fn() -> () constraint
            let _ = web_sys::window()
                .and_then(|w| w.open_with_url_and_target(&tweet_url, "_blank").ok());
        })
    };

    /* --- VIEW LOGIC --- */
    let render_content = || {
        if let Some(msg) = &*error {
            html! {
                <div class="error-container">
                    <p>{ msg }</p>
                    <button class="button" onclick={|_| { let _ = web_sys::window().unwrap().location().reload(); }}>
                        { "Retry" }
                    </button>
                </div>
            }
        } else if *is_loading {
            html! { <div class="loader"></div> }
        } else {
            let (display_text, display_author) = match &*selected_quote {
                Some(q) => (q.text.clone(), q.author.clone()),
                None => ("Click 'New Quote' to get started!".into(), "".into()),
            };

            html! {
                <figure>
                    <blockquote class={if display_text.len() > 120 { "quote-text long-quote" } else { "quote-text" }}>
                        <i class="fas fa-quote-left" aria-hidden="true"></i>
                        <span>{ format!(" {}", display_text) }</span>
                    </blockquote>
                    <figcaption class="quote-author">
                        { "— " }
                        <cite>{ if display_author.is_empty() { "Unknown" } else { &display_author } }</cite>
                    </figcaption>
                    <div class="button-container">
                        <button class="twitter-button" onclick={on_tweet} title="Tweet This!">
                            <i class="fab fa-twitter" aria-hidden="true"></i>
                        </button>
                        <button class="button" onclick={on_new_quote}>{ "New Quote" }</button>
                    </div>
                </figure>
            }
        }
    };

    html! {
        <main class="quote-container">
            { render_content() }
        </main>
    }
}
