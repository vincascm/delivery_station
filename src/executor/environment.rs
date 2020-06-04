use std::collections::{hash_map::Iter, HashMap};

use crate::{
    config::{Config, Repository, Step},
    trigger::TriggeredInfo,
};

type Item<'a> = (&'a str, &'a str);
type Map<'a> = &'a HashMap<String, String>;
type MapIter<'a> = Iter<'a, String, String>;

pub struct Environment<'a> {
    global: Option<Map<'a>>,
    repository: Option<Map<'a>>,
    step: Option<Map<'a>>,
    triggered_info: Vec<Item<'a>>,
}

pub struct EnvironmentIter<'a> {
    global: Option<MapIter<'a>>,
    repository: Option<MapIter<'a>>,
    step: Option<MapIter<'a>>,
    triggered_info: std::vec::IntoIter<Item<'a>>,
}

impl<'a> Iterator for EnvironmentIter<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.global
            .as_mut()
            .and_then(Iterator::next)
            .or_else(|| self.repository.as_mut().and_then(Iterator::next))
            .or_else(|| self.step.as_mut().and_then(Iterator::next))
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .or_else(|| self.triggered_info.next())
    }
}

impl<'a> IntoIterator for Environment<'a> {
    type Item = Item<'a>;
    type IntoIter = EnvironmentIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EnvironmentIter {
            global: self.global.map(IntoIterator::into_iter),
            repository: self.repository.map(IntoIterator::into_iter),
            step: self.step.map(IntoIterator::into_iter),
            triggered_info: self.triggered_info.into_iter(),
        }
    }
}

impl<'a> Step {
    pub fn environment(
        &'a self,
        config: &'a Config,
        repository: &'a Repository,
        ti: &'a TriggeredInfo,
    ) -> Environment<'a> {
        let triggered_info: Vec<(&str, &str)> = [
            ("TRIGGERED_INFO_REPOSITORY", Some(ti.repository.as_str())),
            ("TRIGGERED_INFO_BRANCH", ti.branch.as_deref()),
            ("TRIGGERED_INFO_TAG", ti.tag.as_deref()),
            ("TRIGGERED_INFO_STEPS_NAME", ti.steps_name.as_deref()),
        ]
        .iter()
        .filter_map(|(k, v)| match v {
            Some(v) => Some((*k, *v)),
            None => None,
        })
        .collect();

        Environment {
            triggered_info,
            global: config.environment.as_ref(),
            repository: repository.environment.as_ref(),
            step: self.environment.as_ref(),
        }
    }
}
