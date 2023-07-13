# gets ip addresses of all runing instances with the name tag 'randpiper'

# us-east-1 us-east-2 us-west-1 us-west-2 eu-north-1 eu-central-1 ap-northeast-1 ap-southeast-2

for region in us-east-1 us-east-2 us-west-1 us-west-2 eu-north-1 eu-central-1 ap-northeast-1 ap-southeast-2 ; do
  aws ec2 describe-instances \
  --query "Reservations[*].Instances[*].[PublicIpAddress]" \
  --filters 'Name=tag:Name,Values=randpiper' \
  --output=text --region $region
done > scripts/ip_file