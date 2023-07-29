A port of [yew-markdown](https://github.com/rambip/yew-markdown/) using leptos !

# Usage
```rust
use leptos::*;

{
    let (content, set_content) = create_signal(cx, "# Markdown Power !".to_string());
    ...
    view!{cx,
        <Markdown src=content/>
    }
}
```

# Examples
Look at `./examples/`

There is a demo of an interactive editor [here](https://rambip.github.io/yew-markdown/editor)
