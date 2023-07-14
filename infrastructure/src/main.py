from constructs import Construct
from cdktf import App, TerraformStack
from cdktf_cdktf_provider_aws.provider import AwsProvider
from monitoring.ping_metrics import PingMetrics
from ping.index import PingLambda, PingLambdaScheduler, PingTarget
from vpc.index import VPC, Subnet

class PingStack(TerraformStack):
    pingScheduler: list[PingLambdaScheduler]
    pingLambdas: list[PingLambda]
    awsProvider: AwsProvider
    vpc: VPC

    def __init__(self, scope: Construct, name: str, suffix: str, region: str, environment: str, targets: list[PingTarget]) -> None:
        super().__init__(scope, name + suffix)

        self.awsProvider = AwsProvider(self, "AWS", region=region, profile="default")
        self.vpc = VPC(self, "vpc", suffix, region, environment)
        for (i, subnet) in enumerate(self.vpc.subnets):
            self.pingLambda = PingLambda(self,
                                         "ping-lambda" + str(i),
                                         suffix,
                                         region,
                                         environment,
                                         self.vpc.vpc.default_security_group_id,
                                         subnet.subnet.id)

            self.pingSchedulers = PingLambdaScheduler(self,
                                                    "ping-lambda-scheduler-" + str(i),
                                                    suffix,
                                                    region,
                                                    environment,
                                                    targets,
                                                    self.pingLambda)

class MonitoringStack(TerraformStack):
    pingMetrics: PingMetrics
    awsProvider: AwsProvider

    def __init__(self, scope: Construct, name: str, suffix: str, environment: str, regions, targets) -> None:
        super().__init__(scope, name + suffix)

        self.awsProvider = AwsProvider(self, "AWS", region=region, profile="default")

        self.pingMetrics = PingMetrics(self, "ping-metrics", suffix, environment, regions, targets)

app = App()

pingUrls: dict = {"devo": ["https://api-public.sandbox.exchange.coinbase.com/time",
                           "https://api.kraken.com/0/public/Time",
                           "https://api.sandbox.gemini.com/v1/symbols",
                           "https://data-api.binance.vision/api/v3/time"],
                  "prod": ["https://api.exchange.coinbase.com/time",
                           "https://api.kraken.com/0/public/Time",
                           "https://api.gemini.com/v1/symbols",
                           "https://data-api.binance.vision/api/v3/time"]}
pingNames: list[str] = ["coinbase", "kraken", "gemini", "binance"]
environments = ["devo", "prod"]
regions = ["us-east-1",
           "us-east-2",
           "us-west-1",
           "us-west-2",
           "eu-west-1",
           "eu-central-1",
           "ap-southeast-1",
           "ap-northeast-1"]

vpcStacks = []
pingStacks = []
for environment in environments:
    for region in regions:
        suffix = "-" + environment + "-" + region
        pingTargets = [PingTarget(x, y, environment) for (x, y) in zip(pingNames, pingUrls[environment])]
        pingStacks.append(PingStack(app, "ping-stack", suffix, region, environment, pingTargets))

monitoringStacks = []
for environment in environments:
    suffix = "-" + environment
    pingTargets = [PingTarget(x, y, environment) for (x, y) in zip(pingNames, pingUrls[environment])]
    monitoringStacks.append(MonitoringStack(app, "monitoring-stack", suffix, environment, regions, pingTargets))

app.synth()