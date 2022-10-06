use actix_http::body::BoxBody;


use serial_test::serial;
use actix_web::{test, App, error::Error};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use teo::core::graph::Graph;
use serde_json::json;
use teo::app::app::ServerConfiguration;
use teo::app::serve::make_app;
use crate::helpers::{request, request_get, assert_json_response};

async fn app() -> App<impl ServiceFactory<
    ServiceRequest,
    Response = ServiceResponse<BoxBody>,
    Config = (),
    InitError = (),
    Error = Error,
>> {
    let graph = Graph::new(|g| {
        g.data_source().mongodb("mongodb://127.0.0.1:27017/teotest_query_select");
        g.reset_database();
        g.model("Single", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("str", |f| {
                f.required().string();
            });
            m.field("num", |f| {
                f.required().i32();
            });
            m.field("bool", |f| {
                f.required().bool();
            });
        });
        g.model("Nested", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("str", |f| {
                f.required().string();
            });
            m.relation("items", |r| {
                r.vec("Item").fields(vec!["id"]).references(vec!["nestedId"]);
            });
        });
        g.model("Item", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("str", |f| {
                f.required().string();
            });
            m.field("nestedId", |f| {
                f.required().object_id();
            });
            m.relation("nested", |r| {
                r.object("Nested").fields(vec!["nestedId"]).references(vec!["id"]);
            });
        });
        g.model("Apple", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("str", |f| {
                f.required().string();
            });
            m.relation("pears", |r| {
                r.vec("Pear").through("Fruit").local("apple").foreign("pear");
            });
        });
        g.model("Pear", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("str", |f| {
                f.required().string();
            });
            m.relation("apples", |r| {
                r.vec("Apple").through("Fruit").local("pear").foreign("apple");
            });
        });
        g.model("Fruit", |m| {
            m.field("appleId", |f| {
                f.required().object_id();
            });
            m.field("pearId", |f| {
                f.required().object_id();
            });
            m.relation("apple", |r| {
                r.object("Apple").fields(vec!["appleId"]).references(vec!["id"]);
            });
            m.relation("pear", |r| {
                r.object("Pear").fields(vec!["pearId"]).references(vec!["id"]);
            });
            m.primary(vec!["appleId", "pearId"]);
        });

    }).await;
    make_app(graph, ServerConfiguration::default())
}

#[test]
#[serial]
async fn select_keeps_selected_scalar_non_primary_fields_on_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
        "select": {
            "id": true,
            "str": true,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"}
        }
    })).await;
}

#[test]
#[serial]
async fn select_removes_scalar_non_primary_fields_on_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
        "select": {
            "num": false,
            "bool": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_keeps_selected_scalar_non_primary_fields_on_update() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "update", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "id": true,
            "str": true,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"equals": id.as_str().unwrap()},
            "str": {"equals": "scalar"}
        }
    })).await;
}

#[test]
#[serial]
async fn select_removes_scalar_non_primary_fields_on_update() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "update", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "num": false,
            "bool": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_update() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "update", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_keeps_selected_scalar_non_primary_fields_on_delete() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "delete", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "id": true,
            "str": true,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"equals": id.as_str().unwrap()},
            "str": {"equals": "scalar"}
        }
    })).await;
}

#[test]
#[serial]
async fn select_removes_scalar_non_primary_fields_on_delete() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "delete", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "num": false,
            "bool": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_delete() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "delete", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_upsert_actually_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "singles", "upsert", tson!({
        "where": {
            "id": "12345678901234567890abcd",
        },
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
        "update": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_upsert_actually_update() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "upsert", tson!({
        "where": {
            "id": id,
        },
        "create": {},
        "update": {},
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_find_unique() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "findUnique", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_find_first() {
    let app = test::init_service(app().await).await;
    let id = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "findFirst", tson!({
        "where": {
            "id": id,
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "num": {"equals": 2},
            "bool": {"equals": true}
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_find_many() {
    let app = test::init_service(app().await).await;
    let _id1 = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let _id2 = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "findMany", tson!({
        "where": {
            "bool": true,
        },
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "meta": {
            "count": {"equals": 2}
        },
        "data": [
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            },
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            }
        ]
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_create_many() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "singles", "createMany", tson!({
        "create": [{
            "str": "scalar",
            "num": 2,
            "bool": true
        }, {
            "str": "scalar",
            "num": 2,
            "bool": true
        }],
        "select": {
            "id": false,
            "str": false,
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "meta": {
            "count": {"equals": 2}
        },
        "data": [
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            },
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            }
        ]
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_update_many() {
    let app = test::init_service(app().await).await;
    let _id1 = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let _id2 = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "updateMany", tson!({
        "where": {
            "bool": true
        },
        "update": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
        "select": {
            "id": false,
            "str": false
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "meta": {
            "count": {"equals": 2}
        },
        "data": [
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            },
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            }
        ]
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_on_delete_many() {
    let app = test::init_service(app().await).await;
    let _id1 = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let _id2 = request_get(&app, "singles", "create", tson!({
        "create": {
            "str": "scalar",
            "num": 2,
            "bool": true
        },
    }), 200, "data.id").await;
    let res = request(&app, "singles", "deleteMany", tson!({
        "where": {
            "bool": true
        },
        "select": {
            "id": false,
            "str": false
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "meta": {
            "count": {"equals": 2}
        },
        "data": [
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            },
            {
                "num": {"equals": 2},
                "bool": {"equals": true}
            }
        ]
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_of_nested_many_in_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "nesteds", "create", tson!({
        "create": {
            "str": "scalar",
            "items": {
                "createMany": [
                    {
                        "str": "scalar"
                    },
                    {
                        "str": "scalar"
                    }
                ]
            }
        },
        "include": {
            "items": {
                "select": {
                    "id": false,
                    "nestedId": false
                }
            }
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"},
            "items": [
                {
                    "str": {"equals": "scalar"}
                },
                {
                    "str": {"equals": "scalar"}
                }
            ]
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_of_nested_single_in_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "items", "create", tson!({
        "create": {
            "str": "scalar",
            "nested": {
                "create": {
                    "str": "scalar"
                }
            }
        },
        "include": {
            "nested": {
                "select": {
                    "id": false
                }
            }
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"},
            "nestedId": {"is": "objectId"},
            "nested": {
                "str": {"equals": "scalar"}
            }
        }
    })).await;
}

#[test]
#[serial]
async fn select_can_remove_primary_fields_in_the_output_of_nested_joined_many_in_create() {
    let app = test::init_service(app().await).await;
    let res = request(&app, "apples", "create", tson!({
        "create": {
            "str": "scalar",
            "pears": {
                "createMany": [
                    {
                        "str": "scalar"
                    },
                    {
                        "str": "scalar"
                    }
                ]
            }
        },
        "include": {
            "pears": {
                "select": {
                    "id": false
                }
            }
        }
    })).await;
    assert_json_response(res, 200, tson!({
        "data": {
            "id": {"is": "objectId"},
            "str": {"equals": "scalar"},
            "pears": [
                {
                    "str": {"equals": "scalar"}
                },
                {
                    "str": {"equals": "scalar"}
                }
            ]
        }
    })).await;
}
