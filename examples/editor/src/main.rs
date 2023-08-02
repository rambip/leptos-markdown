use leptos::*;
use leptos_markdown::Markdown;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (content, set_content) = create_signal(cx, String::new());
    view!{cx,
        <div style={"display: flex; align-items: top;"}>
            <textarea type="text"
                on:input = move |ev| set_content(event_target_value(&ev))
                prop:value = content
                rows={80} 
                cols={50}
                style="margin: 20px"
            />

            <Markdown src=content wikilinks=true/>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view!{cx, <App/>})
}
