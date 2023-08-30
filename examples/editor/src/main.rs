use leptos::*;
use leptos_markdown::{Markdown, debug::EventInfo};

#[component]
fn RenderZone(cx: Scope, 
              content: ReadSignal<String>,
              wikilinks_enabled: ReadSignal<bool>, 
              hard_breaks_enabled: ReadSignal<bool>,
              debug_mode: ReadSignal<bool>) -> impl IntoView {


    let (debug_info, set_debug_info) = create_signal(cx, Vec::new());
    provide_context(cx, EventInfo(set_debug_info));

    let debug_info_view = move || {
        debug_info()
            .iter()
            .map(|x| view!{cx, <li>{x}</li>})
            .collect_view(cx)
    };

    view!{cx,
        {move || if debug_mode() {
                view!{cx,
                    <ul>{debug_info_view}</ul>
                }.into_view(cx)
            }
            else {
                view!{cx, 
                    <Markdown src=content 
                          wikilinks=wikilinks_enabled
                          hard_line_breaks=hard_breaks_enabled
                    />
                }
            }}
    }
}


#[component]
fn App(cx: Scope) -> impl IntoView {
    let (content, set_content) = create_signal(cx, "**bold**".into());
    let (wikilinks_enabled, set_wikilinks) = create_signal(cx, false);
    let (hard_breaks_enabled, set_hard_breaks) = create_signal(cx, false);
    let (debug_mode, set_debug_mode) = create_signal(cx, false);

    view!{cx,
        <h1>Markdown editor</h1>
        <div style={"display: flex; align-items: top;"}>
            <div style="width:40%">
                <textarea type="text"
                    on:input = move |ev| set_content(event_target_value(&ev))
                    prop:value = content
                    rows={30}
                    style="margin: 10px; width: 80%"
                />
                <div>
                    <label for="wiki">enable wikilinks: </label>
                    <input type="checkbox" id="wiki"
                        on:input=move |e| set_wikilinks(event_target_checked(&e))
                    />
                </div>
                <div>
                    <label for="hardbreaks">convert soft breaks to hard breaks</label>
                    <input type="checkbox" id="hardbreaks"
                        on:input=move |e| set_hard_breaks(event_target_checked(&e))
                    />
                </div>
                <div>
                    <input type="radio" name="debug-switch" id="html-on" checked=""
                           on:input=move |_| set_debug_mode(false)/>
                    <label for="html-on" style="padding-right:15px">html mode</label>
                    <input type="radio" name="debug-switch" id="debug-on"
                           on:input=move |_| set_debug_mode(true)
                    />
                    <label for="debug-on">debug mode</label>
                </div>
            </div>

            <div style="border: 1px solid black; margin: 10px; width: 50%">
                <RenderZone content=content
                            wikilinks_enabled=wikilinks_enabled
                            hard_breaks_enabled=hard_breaks_enabled
                            debug_mode=debug_mode
                />
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view!{cx, <App/>})
}
