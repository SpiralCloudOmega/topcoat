use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    router::{Form, Json, route},
};

#[derive(Serialize, Deserialize)]
struct Test {
    arg: i32,
}

#[route(GET)]
async fn test(Form(test): Form<Test>) -> Result<Json<Test>> {
    Ok(Json(Test { arg: test.arg + 1 }))
}
