use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionToolChoiceOption, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse, ToolChoiceOptions,
    },
};

use crate::{agent::model::Agent, llm::llm_request::LlmRequest};

impl Agent {
    pub async fn think(
        &self,
        client: &async_openai::Client<OpenAIConfig>,
        llm_request: LlmRequest,
    ) -> anyhow::Result<CreateChatCompletionResponse> {
        let request = self.build_chat_request(llm_request)?;
        let response = client.chat().create(request).await?;
        Ok(response)
    }

    fn build_chat_request(
        &self,
        llm_request: LlmRequest,
    ) -> anyhow::Result<CreateChatCompletionRequest> {
        let messages = llm_request.build_message()?;
        let tool_choice = match llm_request.tool_choice {
            crate::llm::llm_request::ToolChoice::Auto => {
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto)
            }
            crate::llm::llm_request::ToolChoice::Required => {
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Required)
            }
        };
        CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .max_tokens(self.max_tokens)
            .tools(self.tool_definitions())
            .tool_choice(tool_choice)
            .messages(messages)
            .build()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        agent::{
            event::{ContentItem, Message},
            model::Agent,
        },
        llm::llm_request::LlmRequest,
        tool::ToolBox,
    };

    fn agent(model: &str) -> Agent {
        Agent::new(
            model.to_owned(),
            vec!["system".to_owned()],
            Arc::new(ToolBox::default()),
        )
    }

    fn user_request() -> LlmRequest {
        LlmRequest::new(
            Vec::new(),
            vec![ContentItem::Message(Message::new(
                "user".to_owned(),
                "hello".to_owned(),
            ))],
        )
    }

    #[test]
    fn request_uses_configured_model() {
        let request = agent("custom/model")
            .build_chat_request(user_request())
            .expect("request should build");

        assert_eq!(request.model, "custom/model");
        assert_eq!(
            serde_json::to_value(&request).expect("request should serialize")["max_tokens"],
            2048
        );
        assert_eq!(request.messages.len(), 1);
    }

    #[test]
    fn request_uses_configured_max_tokens() {
        let request = agent("custom/model")
            .with_max_tokens(512)
            .build_chat_request(user_request())
            .expect("request should build");

        assert_eq!(
            serde_json::to_value(&request).expect("request should serialize")["max_tokens"],
            512
        );
    }
}
