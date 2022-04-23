use common::data;
use yew::prelude::*;
use crate::prelude::*;



#[function_component(TagBar)]
pub fn tag_bar(tags: &properties::Tags) -> Html {
    let tags = tags.tags.iter().map(|tag| {
        html!{<div class="tag">{&tag.tag}</div>}
    });
    html!{
        <div class="tags">
            {for tags}
        </div>
    }
}