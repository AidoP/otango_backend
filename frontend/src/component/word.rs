use common::data;
use yew::prelude::*;
use crate::prelude::*;

pub struct Word {
    word: LazyData<data::Word>
}
impl Component for Word {
    type Message = LazyData<data::Word>;
    type Properties = properties::Word;

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.word = msg;
        true
    }

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link().clone();
        let url = format!("単語/{}", urlencoding::encode(&ctx.props().word));
        Self {
            word: LazyData::load(&url, move |data| scope.send_message(data))
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <div class="word popup">
                {self.word.map(
                    || html!{<h1><div class="placeholder-text loader" /></h1>},
                    |w| html!{<h1>{&w.word}</h1>},
                    |_| html!{<h1 class="error">{"Not Found"}</h1>}
                )}
                {self.word.map_ok_either(
                    |class| html!{<div class="tags"><div class={class} /></div>},
                    |w| html!{<component::TagBar tags={w.tags.clone()} />},
                    "placeholder-text loader", "placeholder-text loader error"
                )}
                
                {self.word.map_ok_either(
                    |class| vec![html!{
                        <div>
                            <h2 class="reading"><div class={class} /></h2>
                            <ul>
                                <li><div class={class} /></li>
                            </ul>
                        </div>
                    }],
                    |w| w.readings.iter().map(|reading| html!{
                        <div>
                            <h2 class="reading">{&reading.full}</h2>
                            <ul>{for reading.definitions.iter().map(|i| html!{<li>{i}</li>})}</ul>
                        </div>
                    }).collect(),
                    "placeholder-text loader", "placeholder-text loader error"
                )}
            </div>
        }
    }
}