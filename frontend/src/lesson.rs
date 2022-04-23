use common::data;
use yew::prelude::*;
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    None,
    Correct,
    Incorrect,
    Error
}

pub struct Lesson {
    status: Status
}
impl Component for Lesson {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            status: Status::Correct
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        ctx.link().context::<Status>(Callback::noop());
        html!{
            <div>
                {match self.status {
                    Status::None => Html::default(),
                    Status::Correct => html!{
                        <div id="status"><div>{"はい！"}</div></div>
                    },
                    Status::Incorrect => html!{
                        <div id="status"><div>{"いいえ。"}</div></div>
                    },
                    Status::Error => html!{
                        <div id="error-status">
                            <h1>{"Lost Connection"}</h1>
                            <div class="pulser" />
                        </div>
                    }
                }}
            </div>
        }
    }
}