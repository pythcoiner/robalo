use std::fmt::Display;

use serde_json::{Map, Value};

use crate::error::Error;

#[derive(Debug)]
pub enum Action {
    IssueCreated {
        title: String,
        project: String,
        time: String,
        level: String,
    },
    Unknown,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::IssueCreated {
                title,
                project,
                time,
                level,
            } => {
                write!(
                    f,
                    "{}: issue `{}` created at {} ({})",
                    project, title, time, level
                )
            }
            Action::Unknown => write!(f, "Unknown action!"),
        }
    }
}

pub struct Body(Value);

impl Body {
    pub fn new(value: Value) -> Self {
        Self(value)
    }

    pub fn action_str(&self) -> Option<&str> {
        self.0.get("action")?.as_str()
    }

    pub fn data(&self) -> Option<&Map<String, Value>> {
        if let Some(Value::Object(o)) = self.0.get("data") {
            Some(o)
        } else {
            None
        }
    }

    pub fn issue_map(&self) -> Option<&Map<String, Value>> {
        self.data()?.get("issue").and_then(|issue| {
            if let Value::Object(o) = issue {
                Some(o)
            } else {
                None
            }
        })
    }

    pub fn is_issue(&self) -> bool {
        self.data()
            .map(|d| d.contains_key("issue"))
            .unwrap_or(false)
    }

    pub fn is_issue_created(&self) -> bool {
        self.is_issue() && self.action_str().map(|a| a == "created").unwrap_or(false)
    }

    pub fn to_issue_created(&self) -> Result<Action, Error> {
        if let Some(map) = self.issue_map() {
            let title = map
                .get("title")
                .ok_or(Error::MissingField("issue::title"))?
                .as_str()
                .ok_or(Error::FieldType("issue::title"))?
                .into();
            let project = map
                .get("project")
                .ok_or(Error::MissingField("issue::project"))?
                .get("name")
                .ok_or(Error::MissingField("issue::project::name"))?
                .as_str()
                .ok_or(Error::FieldType("issue::project"))?
                .into();
            let time = map
                .get("lastSeen")
                .ok_or(Error::MissingField("issue::lastSeen"))?
                .as_str()
                .ok_or(Error::FieldType("issue::lastSeen"))?
                .into();
            let level = map
                .get("level")
                .ok_or(Error::MissingField("issue::level"))?
                .as_str()
                .ok_or(Error::FieldType("issue::level"))?
                .into();

            Ok(Action::IssueCreated {
                title,
                project,
                time,
                level,
            })
        } else {
            Err(Error::NotAction("issue_created"))
        }
    }

    pub fn action(&self) -> Result<Action, Error> {
        tracing::debug!("Body.action(): self.data={:#?}", self.data());
        if self.is_issue_created() {
            self.to_issue_created()
        } else {
            Ok(Action::Unknown)
        }
    }
}
