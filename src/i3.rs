//! Conversa com o i3 pelo i3-msg: quais workspaces existem e ir para um deles.

use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub name: String,
    pub focused: bool,
}

/// Lê a resposta de `i3-msg -t get_workspaces`.
pub fn parse_workspaces(json: &str) -> Result<Vec<Workspace>, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("resposta do i3 inválida: {e}"))?;
    let list = value.as_array().ok_or("resposta do i3 não é uma lista")?;
    list.iter()
        .map(|w| {
            let name = w["name"].as_str().ok_or("workspace sem nome")?.to_string();
            let focused = w["focused"].as_bool().unwrap_or(false);
            Ok(Workspace { name, focused })
        })
        .collect()
}

/// Um workspace garantidamente vazio: no i3 um workspace só existe com janelas
/// ou em foco, então qualquer número de 1 a 10 fora da lista está vazio.
pub fn pick_empty(existing: &[Workspace]) -> String {
    let taken = |name: &str| existing.iter().any(|w| w.name == name);
    (1..=10)
        .map(|n| n.to_string())
        .chain(std::iter::once("rustwallp-teste".to_string()))
        .chain((2..).map(|n| format!("rustwallp-teste-{n}")))
        .find(|name| !taken(name))
        .expect("sempre há um nome livre")
}

pub fn get_workspaces() -> Result<Vec<Workspace>, String> {
    let out = Command::new("i3-msg")
        .args(["-t", "get_workspaces"])
        .output()
        .map_err(|e| format!("i3-msg não rodou: {e}"))?;
    if !out.status.success() {
        return Err("i3 não respondeu".into());
    }
    parse_workspaces(&String::from_utf8_lossy(&out.stdout))
}

pub fn goto(name: &str) -> Result<(), String> {
    let ok = Command::new("i3-msg")
        .args(["workspace", name])
        .output()
        .map_err(|e| format!("i3-msg não rodou: {e}"))?
        .status
        .success();
    if ok {
        Ok(())
    } else {
        Err(format!("i3 não foi para o workspace {name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws(names: &[&str]) -> Vec<Workspace> {
        names
            .iter()
            .map(|n| Workspace {
                name: n.to_string(),
                focused: false,
            })
            .collect()
    }

    #[test]
    fn parse_ok() {
        let json = r#"[{"id":1,"num":1,"name":"1","visible":true,"focused":false,"urgent":false},
                       {"id":2,"num":2,"name":"2","visible":false,"focused":true,"urgent":false},
                       {"id":3,"num":5,"name":"5","visible":false,"focused":false,"urgent":false}]"#;
        let list = parse_workspaces(json).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(
            list[1],
            Workspace {
                name: "2".into(),
                focused: true
            }
        );
    }

    #[test]
    fn parse_garbage_is_err() {
        assert!(parse_workspaces("not json").is_err());
    }

    #[test]
    fn pick_first_free() {
        assert_eq!(pick_empty(&ws(&["1", "2", "5"])), "3");
    }

    #[test]
    fn pick_named_when_full() {
        let all: Vec<String> = (1..=10).map(|n| n.to_string()).collect();
        let all: Vec<&str> = all.iter().map(String::as_str).collect();
        assert_eq!(pick_empty(&ws(&all)), "rustwallp-teste");
    }

    #[test]
    fn pick_named_2_when_taken() {
        let mut all: Vec<String> = (1..=10).map(|n| n.to_string()).collect();
        all.push("rustwallp-teste".into());
        let all: Vec<&str> = all.iter().map(String::as_str).collect();
        assert_eq!(pick_empty(&ws(&all)), "rustwallp-teste-2");
    }
}
