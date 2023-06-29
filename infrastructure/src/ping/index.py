from cdktf import AssetType, TerraformAsset
from constructs import Construct
from cdktf_cdktf_provider_aws.iam_role import IamRole
from cdktf_cdktf_provider_aws.iam_role_policy_attachment import IamRolePolicyAttachment
from cdktf_cdktf_provider_aws.lambda_function import LambdaFunction
from cdktf_cdktf_provider_aws.lambda_permission import LambdaPermission
from cdktf_cdktf_provider_aws.s3_bucket import S3Bucket
from cdktf_cdktf_provider_aws.s3_object import S3Object
from cdktf_cdktf_provider_aws.scheduler_schedule import SchedulerSchedule, SchedulerScheduleTarget, SchedulerScheduleTargetDeadLetterConfig
from cdktf_cdktf_provider_aws.sqs_queue import SqsQueue
import json

LAMBDA_ROLE_POLICY: str = '{"Version": "2012-10-17","Statement": [{"Action": "sts:AssumeRole","Principal": {"Service": "scheduler.amazonaws.com"},"Effect": "Allow","Sid": ""},{"Action": "sts:AssumeRole","Principal": {"Service": "lambda.amazonaws.com"},"Effect": "Allow","Sid": ""}]}'

class PingTarget:
    dict: dict[str, str]

    def __init__(self, name: str, url: str, environment: str) -> None:
        self.dict = {"name" : name, "url" : url, "environment" : environment}
    
    def getInput(self) -> str:
        return json.dumps(self.dict)


class PingLambda(Construct):
    lambdaFunction: LambdaFunction

    def __init__(self, scope: Construct, name: str, suffix: str, region: str, environment: str) -> None:
        super().__init__(scope, name + suffix)
        self.name = name
        self.environment = environment
        self.region = region

        asset: TerraformAsset = TerraformAsset(scope,
                                               name + suffix + "-asset",
                                               path = "../build/ping",
                                               type = AssetType.ARCHIVE)

        bucket: S3Bucket = S3Bucket(scope,
                                    name + suffix + "-bucket",
                                    bucket = name + suffix)

        lambdaArchive: S3Object = S3Object(scope,
                                           name + suffix + "-bucket-object",
                                           bucket = bucket.bucket,
                                           key = asset.file_name,
                                           source = asset.path)

        role: IamRole = IamRole(scope,
                                name + suffix + "-lambda-exec",
                                assume_role_policy = LAMBDA_ROLE_POLICY)

        rolePolicyAttachment: IamRolePolicyAttachment = IamRolePolicyAttachment(scope,
                                                                                name + suffix + "-lambda-role-managed-policy",
                                                                                policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
                                                                                role = role.name)
        lambdaRolePolicyAttachment: IamRolePolicyAttachment = IamRolePolicyAttachment(scope,
                                                                                      name + suffix + "-lambda-lambda-managed-policy",
                                                                                      policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaRole",
                                                                                      role = role.name)
        cloudWatchRolePolicyAttachment: IamRolePolicyAttachment = IamRolePolicyAttachment(scope,
                                                                                          name + suffix + "-lambda-cloudwatch-managed-policy",
                                                                                          policy_arn = "arn:aws:iam::aws:policy/CloudWatchFullAccess",
                                                                                          role = role.name)
        sqsRolePolicyAttachment: IamRolePolicyAttachment = IamRolePolicyAttachment(scope,
                                                                                   name + suffix + "-lambda-sqs-managed-policy",
                                                                                   policy_arn = "arn:aws:iam::aws:policy/AmazonSQSFullAccess",
                                                                                   role = role.name)
        
        self.lambdaFunction: LambdaFunction = LambdaFunction(scope,
                                             name + suffix + "-lambda",
                                             function_name = name + suffix + "-lambda",
                                             s3_bucket = bucket.bucket,
                                             s3_key = lambdaArchive.key,
                                             handler = "ping.lambda_handler",
                                             runtime = "python3.9",
                                             architectures = ["arm64"],
                                             role = role.arn,
                                             source_code_hash = asset.asset_hash)

class PingLambdaScheduler(Construct):
    schedules: list[SchedulerSchedule]

    def __init__(self, scope: Construct, name: str, suffix: str, region: str, environment: str, pingTargets: list[PingTarget], pingLambda: PingLambda) -> None:
        super().__init__(scope, name + suffix)

        self.schedules = []
        for pingTarget in pingTargets:

            queue: SqsQueue = SqsQueue(scope,
                                       name + suffix + "-" + pingTarget.dict["name"] + "-dlq")
            dlq: SchedulerScheduleTargetDeadLetterConfig = SchedulerScheduleTargetDeadLetterConfig(arn = queue.arn)

            target: SchedulerScheduleTarget = SchedulerScheduleTarget(arn = pingLambda.lambdaFunction.arn,
                                                                      role_arn = pingLambda.lambdaFunction.role,
                                                                      input = pingTarget.getInput(),
                                                                      dead_letter_config = dlq)

            schedule: SchedulerSchedule = SchedulerSchedule(scope,
                                                            name + suffix + "-" + pingTarget.dict["name"] + "-schedule",
                                                            flexible_time_window = {"mode" : "OFF"},
                                                            schedule_expression = "rate(5 minutes)",
                                                            target = target)
            
            self.schedules.append(schedule)

            lambdaPermission: LambdaPermission = LambdaPermission(scope,
                                                                  name + suffix + "-" + pingTarget.dict["name"] + "-lambda-permission",
                                                                  function_name = pingLambda.lambdaFunction.function_name,
                                                                  principal = "scheduler.amazonaws.com",
                                                                  action = "lambda:InvokeFunction",
                                                                  source_arn = schedule.arn)