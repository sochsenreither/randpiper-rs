# Do the setup on the AWS Server

FILE="${1:-/dev/stdin}"
IPS_FILE=${2:-"scripts/aws/ips_file.log"}
IPS=()

# Create IP files
# bash scripts/aws/make_ip_files.sh $FILE $IPS_FILE

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

for ip in "${IPS[@]}"
do
    echo $ip
    ssh -i "randpiper.pem" -o "StrictHostKeyChecking no" -t ubuntu@$ip 'bash -ls' < scripts/aws/setup.sh 
done

# wait

for ip in "${IPS[@]}"
do
  ssh -i "randpiper.pem" ubuntu@$ip "cd randpiper-rs; cat > ips_file" < $IPS_FILE
done
