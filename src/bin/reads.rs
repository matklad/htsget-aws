// AWS Lambda Rust runtime
use lambda_http::{lambda, IntoResponse, Request};
use lambda_runtime::{error::HandlerError, Context};
use serde_json::json;
use uuid::Uuid;

// Rusoto
use rusoto_core::Region;
use rusoto_athena::*;

fn main() {
    // Init env logger for debugging: https://www.rusoto.org/debugging.html
    let _ = env_logger::try_init();

    lambda!(handler)
}

fn athena_query(query: String) {
    //let creds = rusoto_core::DefaultCredentialsProvider(); 
    // XXX: InstanceProfile creds since this is a lambda instead of hardcoding region
    let client = AthenaClient::new(Region::ApSoutheast2);
    let request_token = Uuid::new_v4();

    let query_input = StartQueryExecutionInput {
        client_request_token: Some(request_token.to_string()),
        query_string: query,
        query_execution_context: Default::default(),
        result_configuration: Default::default(),
        work_group: Default::default()
    };

    match client.start_query_execution(query_input).sync() {
        Ok(output) => {
            match output.query_execution_id {
                Some(query_id) => println!("query running. id: {}", query_id),
                None => println!("query running. no id found"),
            } 
        },
        Err(error) => {
            println!("Error: {:?}", error);
        },
    }
}

fn http_request_to_athena_query(uri_id: String) -> String {
    // A string identifying which records to return.
    
    // The format of this identifier is left to the discretion of the 
    // API provider, including allowing embedded “/” characters. 
    // The following would be valid identifiers:
    
    // ReadGroupSetIds or VariantSetIds as defined by the GA4GH API
    // Studies: PRJEB4019 or /byStudy/PRJEB4019
    // Samples: NA12878 or /data/platinum/NA12878
    // Runs: ERR148333 or /byRun/ERR148333

    let split = uri_id.split("/");
    let mut path_parts: Vec<&str> = split.collect();

    // XXX: only return last element for now, be more clever afterwards?
    let id = path_parts.pop().unwrap();

    // XXX: Parametrize for start/end and many others according to spec and backend schema
    let sql_query = format!("SELECT referencename FROM htsget.adam WHERE referencename LIKE '{}';", id);

    return sql_query;
}

fn handler(
    req: Request,
    _: Context,
) -> Result<impl IntoResponse, HandlerError> {
    // `serde_json::Values` impl `IntoResponse` by default
    // creating an application/json response

    let id_query = http_request_to_athena_query(req.uri().path().to_string());
    athena_query(id_query);

//{"errorMessage":"JsonError: invalid type: null, expected a HeaderMap<HeaderValue> at line 1 column 84","errorType":"JsonError","stackTrace":null}

    Ok(json!({
        "message": "Reads: Your function executed successfully!"
    }))
}