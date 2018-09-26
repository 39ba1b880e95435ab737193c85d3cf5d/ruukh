//! Representation of text/comment in virtual dom tree.

use crate::{
    component::Render,
    dom::{DOMInfo, DOMPatch, DOMRemove, DOMReorder},
    vdom::VNode,
    web_api::*,
    MessageSender, Shared,
};
use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};
use wasm_bindgen::prelude::JsValue;

/// The representation of text/comment in virtual dom tree.
pub struct VText<RCTX: Render> {
    /// The content of a text string
    content: String,
    /// Whether the content is a comment
    is_comment: bool,
    /// Text/Comment reference to the DOM
    node: Option<Node>,
    /// Render context
    _phantom: PhantomData<RCTX>,
}

impl<RCTX: Render> VText<RCTX> {
    /// Create a textual VText.
    pub fn text<T: Into<String>>(content: T) -> VText<RCTX> {
        VText {
            content: content.into(),
            is_comment: false,
            node: None,
            _phantom: PhantomData,
        }
    }

    /// Create a comment VText.
    pub fn comment<T: Into<String>>(content: T) -> VText<RCTX> {
        VText {
            content: content.into(),
            is_comment: true,
            node: None,
            _phantom: PhantomData,
        }
    }
}

impl<RCTX: Render> From<VText<RCTX>> for VNode<RCTX> {
    fn from(text: VText<RCTX>) -> VNode<RCTX> {
        VNode::Text(text)
    }
}

impl<RCTX: Render> Display for VText<RCTX> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_comment {
            write!(f, "<!--{}-->", self.content)
        } else {
            write!(f, "{}", self.content)
        }
    }
}

impl<RCTX: Render> VText<RCTX> {
    fn patch_new(&mut self, parent: &Node, next: Option<&Node>) -> Result<(), JsValue> {
        let node: Node = if self.is_comment {
            document.create_comment(&self.content).into()
        } else {
            document.create_text_node(&self.content).into()
        };

        if let Some(next) = next {
            parent.insert_before(&node, next)?;
        } else {
            parent.append_child(&node)?;
        }
        self.node = Some(node);
        Ok(())
    }
}

impl<RCTX: Render> DOMPatch<RCTX> for VText<RCTX> {
    type Node = Node;

    fn render_walk(
        &mut self,
        _: &Node,
        _: Option<&Node>,
        _: Shared<RCTX>,
        _: MessageSender,
    ) -> Result<(), JsValue> {
        unreachable!("There is nothing to render in a VText");
    }

    fn patch(
        &mut self,
        old: Option<&mut Self>,
        parent: &Node,
        next: Option<&Node>,
        _: Shared<RCTX>,
        _: MessageSender,
    ) -> Result<(), JsValue> {
        if let Some(old) = old {
            if self.is_comment == old.is_comment {
                let old_node = old
                    .node
                    .as_ref()
                    .expect("The old node is expected to be attached to the DOM");
                if self.content != old.content {
                    old_node.set_text_content(&self.content);
                }
                self.node = Some(old_node.clone());
                Ok(())
            } else {
                old.remove(parent)?;
                self.patch_new(parent, next)
            }
        } else {
            self.patch_new(parent, next)
        }
    }
}

impl<RCTX: Render> DOMReorder for VText<RCTX> {
    fn reorder(&self, parent: &Node, next: Option<&Node>) -> Result<(), JsValue> {
        let node = self.node.as_ref().unwrap();
        if let Some(next) = next {
            parent.insert_before(node, next)?;
        } else {
            parent.append_child(node)?;
        }
        Ok(())
    }
}

impl<RCTX: Render> DOMRemove for VText<RCTX> {
    type Node = Node;

    fn remove(&self, parent: &Node) -> Result<(), JsValue> {
        parent.remove_child(
            self.node
                .as_ref()
                .expect("The old node is expected to be attached to the DOM"),
        )?;
        Ok(())
    }
}

impl<RCTX: Render> DOMInfo for VText<RCTX> {
    fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::component::root_render_ctx;
    use wasm_bindgen_test::*;

    #[test]
    fn should_display_text() {
        let text = VText::<()>::text("This is a very fine day!");
        assert_eq!(format!("{}", text), "This is a very fine day!");
    }

    #[test]
    fn should_display_comment() {
        let comment = VText::<()>::comment("Something to remind the hacky users.");
        assert_eq!(
            format!("{}", comment),
            "<!--Something to remind the hacky users.-->"
        );
    }

    #[wasm_bindgen_test]
    fn should_patch_container_with_new_text() {
        let mut vtext = VText::text("Hello World! It is nice to render.");
        let div = html_document.create_element("div").unwrap();
        vtext
            .patch(
                None,
                div.as_ref(),
                None,
                root_render_ctx(),
                crate::message_sender(),
            ).expect("To patch the div");

        assert_eq!(div.inner_html(), "Hello World! It is nice to render.");
    }

    #[wasm_bindgen_test]
    fn should_patch_container_with_text_update() {
        let mut vtext = VText::text("Hello World! It is nice to render.");
        let div = html_document.create_element("div").unwrap();
        vtext
            .patch(
                None,
                div.as_ref(),
                None,
                root_render_ctx(),
                crate::message_sender(),
            ).expect("To patch div");

        assert_eq!(div.inner_html(), "Hello World! It is nice to render.");

        let mut updated = VText::text("How you doing?");
        updated
            .patch(
                Some(&mut vtext),
                div.as_ref(),
                None,
                root_render_ctx(),
                crate::message_sender(),
            ).expect("To patch div");

        assert_eq!(div.inner_html(), "How you doing?");
    }

    #[wasm_bindgen_test]
    fn should_patch_container_with_new_comment() {
        let mut comment = VText::comment("This is a comment");
        let div = html_document.create_element("div").unwrap();
        comment
            .patch(
                None,
                div.as_ref(),
                None,
                root_render_ctx(),
                crate::message_sender(),
            ).expect("To patch div");

        assert_eq!(div.inner_html(), "<!--This is a comment-->");
    }

    #[wasm_bindgen_test]
    fn should_patch_container_with_new_text_on_comment() {
        let mut comment = VText::comment("This is a comment");
        let div = html_document.create_element("div").unwrap();
        comment
            .patch(
                None,
                div.as_ref(),
                None,
                root_render_ctx(),
                crate::message_sender(),
            ).expect("To patch div");

        assert_eq!(div.inner_html(), "<!--This is a comment-->");

        let mut text = VText::text("This is a text");
        text.patch(
            Some(&mut comment),
            div.as_ref(),
            None,
            root_render_ctx(),
            crate::message_sender(),
        ).expect("To patch div");

        assert_eq!(div.inner_html(), "This is a text");
    }

}
