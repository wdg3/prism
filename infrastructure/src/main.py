from constructs import Construct
from cdktf import App, TerraformStack
from cdktf_cdktf_provider_aws.provider import AwsProvider
from ping.index import PingLambda, PingLambdaScheduler, PingTarget

class PingStack(TerraformStack):
    pingScheduler: PingLambdaScheduler
    pingLambda: PingLambda
    awsProvider: AwsProvider

    def __init__(self, scope: Construct, name: str, suffix: str, region: str, environment: str, targets: list[PingTarget]) -> None:
        super().__init__(scope, name + suffix)

        self.awsProvider = AwsProvider(self, "AWS", region=region, profile="default")

        self.pingLambda = PingLambda(self, "ping-lambda", suffix, region, environment)

        self.pingSchedulers = PingLambdaScheduler(self,
                                                  "ping-lambda-scheduler",
                                                  suffix,
                                                  region,
                                                  environment,
                                                  targets,
                                                  self.pingLambda)

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

pingStacks = []
for environment in ["devo", "prod"]:
    for region in ["us-east-1",
                   "us-east-2",
                   "us-west-1",
                   "us-west-2",
                   "eu-west-1",
                   "eu-central-1",
                   "ap-southeast-1",
                   "ap-northeast-1"]:
        suffix = "-" + environment + "-" + region
        pingTargets = [PingTarget(x, y, environment) for (x, y) in zip(pingNames, pingUrls[environment])]
        pingStacks.append(PingStack(app, "ping-stack", suffix, region, environment, pingTargets))

app.synth()