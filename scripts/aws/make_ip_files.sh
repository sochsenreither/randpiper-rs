IPS_FILE=${2:-"scripts/aws/ips_file.log"}
IPS=()
BASE_PORT=7000
CLIENT_PORT=6000

if [ -e "$IPS_FILE" ]; then
    echo "File [$IPS_FILE] already exists"
    rm -rf $IPS_FILE
fi

idx=0

while IFS= read -r line; do
    IPS+=($line":"$(($BASE_PORT+$idx)))
done

for i in ${IPS[@]}; do
    echo "$i" >> $IPS_FILE
done
