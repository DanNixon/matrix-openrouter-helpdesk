use handlebars::Handlebars;
use miette::IntoDiagnostic;
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

    pub fn render_reply(&self, template: &str, response: &str) -> miette::Result<String> {
        let data = json!({
            "response": response,
        });

        self.handlebars
            .render_template(template, &data)
            .into_diagnostic()
    }
}
