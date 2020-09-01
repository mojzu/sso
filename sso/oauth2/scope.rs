/// Scope
#[derive(Debug, Default, Clone)]
pub struct Scope(Vec<String>);

impl Scope {
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }

    pub fn from_ref<T: AsRef<str>>(scope: &[T]) -> Self {
        Self(scope.iter().map(|x| x.as_ref().to_string()).collect())
    }

    pub fn from_string<T: AsRef<str>>(scope: T) -> Self {
        Self(
            scope
                .as_ref()
                .split(" ")
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect(),
        )
    }

    pub fn to_string(&self) -> String {
        self.0.join(" ")
    }

    pub fn from_to_string<T: AsRef<str>>(scope: T) -> String {
        Self::from_string(scope).to_string()
    }

    pub fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, scope: &Self) -> bool {
        for value in scope.as_ref().iter() {
            if !self.0.contains(value) {
                return false;
            }
        }
        true
    }
}

impl<T: AsRef<str>> From<Vec<T>> for Scope {
    fn from(x: Vec<T>) -> Self {
        Self::from_ref(&x)
    }
}

impl From<String> for Scope {
    fn from(x: String) -> Self {
        Self::from_string(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope() {
        let scope_v = Scope::from_ref(&["foo", "bar", "baz"]);
        assert_eq!(scope_v.to_string(), "foo bar baz");

        let scope_s = Scope::from_string(" foo  bar  baz  ");
        assert_eq!(scope_s.to_string(), "foo bar baz");

        assert!(scope_v.contains(&scope_s));
    }
}
