use crate::error::{HelpdeskError, Result};
use handlebars::Handlebars;
use serde_json::json;

pub struct TemplateRenderer {
    handlebars: Handlebars<'static>,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        Self {
            handlebars: Handlebars::new(),
        }
    }

    pub fn render_question(&self, template: &str, query: &str) -> Result<String> {
        let data = json!({
            "query": query,
        });
        self.handlebars
            .render_template(template, &data)
            .map_err(|e| HelpdeskError::Template(e.to_string()).into())
    }

    pub fn render_reply(&self, template: &str, response: &str) -> Result<String> {
        let data = json!({
            "response": response,
        });
        self.handlebars
            .render_template(template, &data)
            .map_err(|e| HelpdeskError::Template(e.to_string()).into())
    }
}
