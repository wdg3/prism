import boto3
import requests

def lambda_handler(event, context):
    cloudwatch = boto3.client("cloudwatch")

    url: str = event["url"]
    
    response: requests.Response = requests.get(url)

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