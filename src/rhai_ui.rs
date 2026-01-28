use dioxus::prelude::*;
use rhai::{Engine, Scope, Map, Array, Dynamic};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum UiNode {
    Element {
        tag: String,
        props: HashMap<String, String>,
        children: Vec<UiNode>,
    },
    Text(String),
}

impl UiNode {
    pub fn from_dynamic(val: Dynamic) -> Result<Self, String> {
        if val.is_string() {
            return Ok(UiNode::Text(val.into_string().unwrap()));
        }

        let map = val.try_cast::<Map>().ok_or("Expected Map for Element")?;

        let tag = map.get("tag")
            .ok_or("Missing 'tag'")?
            .clone()
            .into_string()
            .map_err(|_| "tag must be string")?;

        let props_dyn = map.get("props")
            .ok_or("Missing 'props'")?
            .clone()
            .try_cast::<Map>()
            .ok_or("props must be map")?;

        let mut props = HashMap::new();
        for (k, v) in props_dyn {
            props.insert(k.into(), v.to_string());
        }

        let children_dyn = map.get("children")
            .ok_or("Missing 'children'")?
            .clone()
            .try_cast::<Array>()
            .ok_or("children must be array")?;

        let mut children = Vec::new();
        for child in children_dyn {
            children.push(UiNode::from_dynamic(child)?);
        }

        Ok(UiNode::Element { tag, props, children })
    }
}

pub fn create_rhai_engine() -> Engine {
    let mut engine = Engine::new();

    engine.register_fn("el", |tag: &str, props: Map, children: Array| -> Map {
        let mut map = Map::new();
        map.insert("tag".into(), tag.into());
        map.insert("props".into(), props.into());
        map.insert("children".into(), children.into());
        map
    });

    engine.register_fn("text", |s: &str| -> String {
        s.to_string()
    });

    engine.register_fn("v", |arr: Array| -> Array {
        arr
    });

    engine
}

#[component]
pub fn RhaiRenderer(script: String, context: String) -> Element {
    let node = use_memo(move || {
        let engine = create_rhai_engine();
        let mut scope = Scope::new();

        // Parse context JSON and add to scope
        if let Ok(ctx_val) = serde_json::from_str::<serde_json::Value>(&context) {
             let dynamic_ctx = rhai::serde::to_dynamic(&ctx_val).unwrap_or(Dynamic::UNIT);
             scope.push("data", dynamic_ctx);
        }

        match engine.eval_with_scope::<Dynamic>(&mut scope, &script) {
            Ok(result) => UiNode::from_dynamic(result),
            Err(e) => Err(e.to_string()),
        }
    });

    let current_node = node.read();
    match &*current_node {
        Ok(root) => rsx! { RenderUiNode { node: root.clone() } },
        Err(e) => rsx! {
            div {
                class: "text-red-500 p-4 border border-red-500 rounded bg-red-50",
                "Error rendering Rhai UI: {e}"
            }
        }
    }
}

#[component]
fn RenderUiNode(node: UiNode) -> Element {
    match node {
        UiNode::Text(t) => rsx! { "{t}" },
        UiNode::Element { tag, props, children } => {
            let class = props.get("class").cloned().unwrap_or_default();
            // Basic event handlers (optional, for now just static)

            match tag.as_str() {
                "div" => rsx! {
                    div { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "span" => rsx! {
                    span { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "h1" => rsx! {
                    h1 { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "h2" => rsx! {
                    h2 { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "p" => rsx! {
                    p { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "button" => rsx! {
                    button { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "ul" => rsx! {
                    ul { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "li" => rsx! {
                    li { class: "{class}",
                        {children.into_iter().map(|child| rsx! { RenderUiNode { node: child } })}
                    }
                },
                "img" => rsx! {
                    img {
                        class: "{class}",
                        src: props.get("src").cloned().unwrap_or_default(),
                        alt: props.get("alt").cloned().unwrap_or_default(),
                    }
                },
                 "input" => rsx! {
                    input {
                        class: "{class}",
                        value: props.get("value").cloned().unwrap_or_default(),
                        r#type: props.get("type").cloned().unwrap_or("text".to_string()),
                    }
                },
                _ => rsx! {
                    div { class: "text-orange-500", "Unknown tag: {tag}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::{Dynamic, Scope};

    #[test]
    fn test_rhai_script_rendering() {
        let engine = create_rhai_engine();
        let mut scope = Scope::new();
        let script = r#"
            el("div", #{ "class": "container" }, [
                el("h1", #{}, [ text("Hello") ])
            ])
        "#;

        let result = engine.eval_with_scope::<Dynamic>(&mut scope, script).unwrap();
        let ui_node = UiNode::from_dynamic(result).unwrap();

        match ui_node {
            UiNode::Element { tag, props, children } => {
                assert_eq!(tag, "div");
                assert_eq!(props.get("class").unwrap(), "container");
                assert_eq!(children.len(), 1);

                match &children[0] {
                    UiNode::Element { tag, children, .. } => {
                        assert_eq!(tag, "h1");
                        assert_eq!(children.len(), 1);
                        match &children[0] {
                            UiNode::Text(t) => assert_eq!(t, "Hello"),
                            _ => panic!("Expected text"),
                        }
                    },
                    _ => panic!("Expected h1"),
                }
            },
            _ => panic!("Expected div"),
        }
    }
}
