//! Create and message ordinary project chats through the resident server's
//! chat API.

use std::io::Read;

use crate::error::{anyhow, Result};
use crate::local::chat;
use crate::AgentCommand;

pub async fn run(args: crate::AgentArgs) -> Result<()> {
    let port = chat::trusted_up_port()?.ok_or_else(|| anyhow!(
        "Connect to the running OpenResearch server with ORX_LOCAL_SESSION=1 and ORX_UP_PORT (and ORX_UP_AUTH_TOKEN when required)."
    ))?;
    match args.command {
        AgentCommand::Spawn {
            task,
            stdin,
            title,
            project,
        } => {
            let prompt = message_text(task, stdin, "orx agent spawn \"<message>\"")?;
            let project = match project {
                Some(project) => project,
                None => {
                    let session_id = chat::launching_chat_session().ok_or_else(|| {
                        anyhow!(
                        "Pass --project <project-id> when there is no current OpenResearch chat."
                    )
                    })?;
                    let session = crate::commands::up::chat_request_via_up(
                        port,
                        reqwest::Method::GET,
                        &format!("/api/chat/sessions/{session_id}"),
                        &serde_json::json!({}),
                    )
                    .await?;
                    session
                        .pointer("/session/projectId")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            anyhow!("The current chat is unavailable. Pass --project <project-id>.")
                        })?
                        .to_string()
                }
            };
            spawn(port, &project, &prompt, title.as_deref()).await
        }
        AgentCommand::Send {
            session,
            message,
            stdin,
        } => {
            let prompt = message_text(message, stdin, "orx agent send <chat-id> \"<message>\"")?;
            send(port, &session, &prompt).await
        }
    }
}

fn message_text(task: Option<String>, stdin: bool, usage: &str) -> Result<String> {
    let text = if stdin {
        if task.is_some() {
            return Err(anyhow!(
                "Pass the message as an argument or --stdin, not both."
            ));
        }
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        task.unwrap_or_default()
    };
    let text = text.trim();
    if text.is_empty() {
        return Err(anyhow!("Provide a message: {usage}."));
    }
    Ok(text.to_string())
}

async fn spawn(port: u16, project: &str, prompt: &str, title: Option<&str>) -> Result<()> {
    use crate::commands::up::chat_request_via_up;
    use reqwest::Method;
    use serde_json::json;

    let created = chat_request_via_up(
        port,
        Method::POST,
        "/api/chat/sessions",
        &json!({"projectId": project}),
    )
    .await?;
    let id = created
        .pointer("/session/id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("OpenResearch returned no created chat ID."))?;
    println!("Created chat {id}.");
    println!("Open it in OpenResearch: http://127.0.0.1:{port}/projects/{project}/tasks/{id}");
    if let Some(title) = title.map(str::trim).filter(|title| !title.is_empty()) {
        chat_request_via_up(
            port,
            Method::PATCH,
            &format!("/api/chat/sessions/{id}"),
            &json!({"title": title}),
        )
        .await
        .map_err(|error| {
            anyhow!(
                "Chat {id} was created, but naming it failed: {error}. Open that chat to continue."
            )
        })?;
    }
    let client_turn_id = format!("ct_{}", uuid::Uuid::new_v4());
    let result = chat_request_via_up(port, Method::POST, &format!("/api/chat/sessions/{id}/message"),
        &json!({"text": prompt, "clientTurnId": client_turn_id})).await
        .map_err(|error| anyhow!("Chat {id} was created, but opening-message acceptance could not be confirmed: {error}. Check that chat before retrying."))?;
    if result
        .pointer("/turn/queued")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        println!("Opening message accepted and queued.");
    } else {
        println!("Opening message accepted.");
    }
    Ok(())
}

async fn send(port: u16, session: &str, prompt: &str) -> Result<()> {
    use crate::commands::up::chat_request_via_up;
    use reqwest::Method;
    use serde_json::json;

    let client_turn_id = format!("ct_{}", uuid::Uuid::new_v4());
    let result = chat_request_via_up(
        port,
        Method::POST,
        &format!("/api/chat/sessions/{session}/message"),
        &json!({"text": prompt, "clientTurnId": client_turn_id}),
    )
    .await
    .map_err(|error| anyhow!("Message to chat {session} could not be confirmed: {error}."))?;
    if result
        .pointer("/turn/queued")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        println!("Follow-up accepted and queued for chat {session}.");
    } else {
        println!("Follow-up accepted for chat {session}.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::message_text;
    #[test]
    fn message_requires_one_nonempty_source() {
        let usage = "orx agent send <chat-id> \"<message>\"";
        assert_eq!(
            message_text(Some("  Inspect the code  ".into()), false, usage).unwrap(),
            "Inspect the code"
        );
        assert!(message_text(None, false, usage).is_err());
        assert!(message_text(Some("   ".into()), false, usage).is_err());
        assert!(message_text(Some("argument".into()), true, usage).is_err());
    }
}
