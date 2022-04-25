use common::data;
use yew::prelude::*;
use crate::prelude::*;

pub struct Kanji {
    kanji: LazyData<data::Kanji>
}
impl Component for Kanji {
    type Message = LazyData<data::Kanji>;
    type Properties = properties::Kanji;

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.kanji = msg;
        true
    }

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link().clone();
        let url = format!("漢字/{}", urlencoding::encode(&ctx.props().kanji));
        Self {
            kanji: LazyData::load(&url, move |data| scope.send_message(data))
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <div class="kanji popup">
                {self.kanji.map(
                    || html!{<h1><div class="placeholder-text loader" /></h1>},
                    |k| html!{<h1>{&k.kanji}</h1>},
                    |_| html!{<h1 class="error">{"Error"}</h1>}
                )}
                {self.kanji.map(
                    || html!{<p><div class="placeholder-text loader" /></p>},
                    |k| html!{<p>{&k.memonic}</p>},
                    |e| html!{<p class="error">{e}</p>}
                )}
            </div>
        }
    }
}