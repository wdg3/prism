import boto3
import json
import requests

def lambda_handler(event, context):
    cloudwatch = boto3.client("cloudwatch")

    url: str = event["url"]
    headers: dict[str, str] = {"User-Agent": "python-requests/2.31.0",
                               "Accept-Encoding": "gzip, deflate",
                               "Accept": "*/*",
                               "Connection": "close"}
    response: requests.Response = requests.get(url, headers = headers)

    if (response.status_code == 200):
        cloudwatch.put_metric_data(
            MetricData = [
                {
                    "MetricName": "Latency",
                    "Dimensions": [
                        {
                            "Name": "Target",
                            "Value": event["name"]
                        },
                        {
                            "Name": "Environment",
                            "Value": event["environment"]
                        },
                    ],
                    "Unit": "Milliseconds",
                    "Value": response.elapsed.microseconds // 1000
                }
            ],
            Namespace = "Ping"
        )
    log = {
        "Path" : response.request.url,
        "Request" : response.request.body,
        "Status" : response.status_code,
        "Reason" : response.reason,
        "Response" : response.json(),
        "Latency" : str(response.elapsed.microseconds // 1000) + "ms"
    }
    print(json.dumps(log))