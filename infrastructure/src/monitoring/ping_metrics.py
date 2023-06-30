import json
from constructs import Construct
from cdktf_cdktf_provider_aws.cloudwatch_dashboard import CloudwatchDashboard
from ping.index import PingTarget

class DashboardBody:
    content: dict[str, any]

    def __init__(self):
        self.content = {}
    
    def put(self, k, v):
        self.content[k] = v

    def getContent(self) -> str:
        return json.dumps(self.content)

class PingMetrics(Construct):
    regionDashboard: CloudwatchDashboard
    
    def __init__(self, scope: Construct, name: str, suffix: str, environment: str, regions: list[str], targets: list[PingTarget]) -> None:
        super().__init__(scope, name + suffix)

        regionBody = DashboardBody()
        regionWidgets = []
        for region in regions:
            metrics = []
            for target in targets:
                targetName = target.dict["name"]
                metric = ["Ping", "Latency", "Environment", environment, "Target", targetName]
                metrics.append(metric)
            props = {
                "metrics" : metrics,
                "view": "timeSeries",
                "stacked" : False,
                "period" : 300,
                "stat" : "Average",
                "region" : region,
                "title" : "Exchange Latency" + " - " + region
            }
            widget = {
                "type" : "metric",
                "properties" : props
            }
            regionWidgets.append(widget)
        regionBody.put("widgets", regionWidgets)

        self.regionDashboard: CloudwatchDashboard = CloudwatchDashboard(scope,
                                                                        name + suffix + "-region-dashboard",
                                                                        dashboard_body = regionBody.getContent(),
                                                                        dashboard_name = "PingLatencyByRegion-" + environment)