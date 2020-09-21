/// Scope
#[derive(Debug, Default, Clone)]
pub struct Scope(Vec<String>);

impl Scope {
    /// Returns inner vector
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }

    /// Returns scope from string vector
    pub fn from_ref<T: AsRef<str>>(scope: &[T]) -> Self {
        Self(scope.iter().map(|x| x.as_ref().to_string()).collect())
    }

    /// Returns scope from space separated scope string
    pub fn from_string<T: AsRef<str>>(scope: T) -> Self {
        Self(
            scope
                .as_ref()
                .split(' ')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect(),
        )
    }

    /// Trims extra spaces from scope string
    pub fn from_to_string<T: AsRef<str>>(scope: T) -> String {
        Self::from_string(scope).to_string()
    }

    /// Returns true if scope is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this scope contains scope argument
    pub fn contains(&self, scope: &Self) -> bool {
        for value in scope.as_ref().iter() {
            if !self.0.contains(value) {
                return false;
            }
        }
        true
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

impl AsRef<[String]> for Scope {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
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
