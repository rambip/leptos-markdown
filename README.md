A port of [yew-markdown](https://github.com/rambip/yew-markdown/) using leptos !

# Usage
You can use this component to render both static and dynamic markdown.
## Static markdown

```rust
use leptos::*;

{
    ...
    view!{cx,
        <Markdown src="# Markdown Power !"/>
    }
}
```

## Dynamic markdown
```rust
{
    ...
    let (content, set_content) = create_signal(cx, "# Markdown Power !".to_string());

    view!{cx,
        <Markdown src=content/>
    }
}
```

# Examples
To build them, just follow the [leptos installation instructions](https://leptos-rs.github.io/leptos/02_getting_started.html) and run `trunk serve` to try them.

## Showcase
![](./showcase.jpg)

`./examples/showcase`

You can see the result [here](https://rambip.github.io/leptos-markdown/showcase)

To be fair, this is not the vanilla component, there is a bit of styling added.

## Editor
`./examples/editor`

There is a demo of an interactive editor [here](https://rambip.github.io/leptos-markdown/editor)


# Comparison
This project was great to compare the advantages and drawbacks of the two major rust web frameworks !

see [my feedback](./feedback/README.md) for a comparison

