use yew::prelude::*;
use yew_router::prelude::*;

mod prelude {
    pub(crate) use super::util::LazyData;
    pub(crate) use super::component;
    pub(crate) use super::properties;
}

mod info;
mod properties;
mod component;
mod util;
mod lesson;
use lesson::Lesson;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Lesson,
    #[at("/w/:word")]
    Word { word: String },
    #[at("/k/:kanji")]
    Kanji { kanji: String },
    #[not_found]
    #[at("/not_found")]
    NotFound,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Lesson => html! { <Lesson /> },
        Route::Word { word } => {
            if let Ok(word) = urlencoding::decode(word) {
                html! { <component::Word word={word.to_string()} /> }
            } else {
                html! { <info::InvalidPath /> }
            }
        },
        Route::Kanji { kanji } => {
            if let Ok(word) = urlencoding::decode(kanji) {
                html! { <component::Kanji kanji={word.to_string()} /> }
            } else {
                html! { <info::InvalidPath /> }
            }
        },
        Route::NotFound => html! { <info::NotFound /> },
    }
}

/*
#[function_component(Word)]
fn word(properties: &properties::Word) -> Html {
    let data = use_state(|| component::Word::Loading);
    {
        let data = data.clone();
        let url = format!("https://192.168.1.128:8000/単語/{}", properties.word);
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                data.set(component::Word::get(properties.word));
            });
            || ()
        });
    }
    use std::ops::Deref;
    let data = data.deref();
    html!{
        <pre>{ data }</pre>
    }
}*/

#[function_component(App)]
fn app() -> Html {
    html!{
        <BrowserRouter>
            <header>
                <Link<Route> to={Route::Lesson}>{"Home"}</Link<Route>>
                //<Link<Route> to={Route::Word}>{"Words"}</Link<Route>>
            </header>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}
fn main() {
    yew::Renderer::<App>::new().render();
}
