# Todos
* High priority
  * print time for every message
  * add the possibility to override request and response headers (--request-header name=value, --response-header name=value)
  * in some cases it is hard to know where the error comes from, make error message more verbose
    * create custom errors instead of simple boxing

* Low priority
  * make logging similar in graphql_proxy.rs and graphql_ws_proxy.rs
  * possibility to serve graphiql from local files
  * use restest in tests with parameters
