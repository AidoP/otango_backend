//! Information Pages

use yew::prelude::*;

pub struct NotFound {

}
impl Component for NotFound {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <h1>{ "404 Not Found" }</h1>
        }
    }
}

#[function_component(InvalidPath)]
pub fn invalid_path() -> Html {
    html!{
        <h1>{"Paths must be UTF-8"}</h1>
    }
}