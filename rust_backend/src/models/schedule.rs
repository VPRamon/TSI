
#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ScheduleId(pub i64);

impl std::fmt::Display for ScheduleId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<i64> for ScheduleId {
	fn from(v: i64) -> Self {
		ScheduleId(v)
	}
}

impl From<ScheduleId> for i64 {
	fn from(s: ScheduleId) -> Self {
		s.0
	}
}

impl<'a> tiberius::FromSql<'a> for ScheduleId {
	fn from_sql(value: &'a tiberius::ColumnData<'static>) -> tiberius::Result<Option<Self>> {
		match <i64 as tiberius::FromSql<'a>>::from_sql(value)? {
			Some(v) => Ok(Some(ScheduleId(v))),
			None => Ok(None),
		}
	}
}

impl<'a> tiberius::IntoSql<'a> for ScheduleId {
	fn into_sql(self) -> tiberius::ColumnData<'a> {
		<i64 as tiberius::IntoSql<'a>>::into_sql(self.0)
	}
}

impl tiberius::ToSql for ScheduleId {
	fn to_sql(&self) -> tiberius::ColumnData<'_> {
		<i64 as tiberius::ToSql>::to_sql(&self.0)
	}
}