import json
import random
import time
import subprocess
import datetime
import sys
import uuid

# Event types matches one of the Rust enum definitions
ACTION_TYPES = ["file_integrity", "process_monitor", "network_socket", "system_log"]
SEVERITY_LEVELS = ["INFO", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
HOSTNAMES = ["server-alpha", "server-beta", "workstation-01", "firewall-main", "db-cluster"]
FILES = ["/etc/passwd", "/var/www/html/index.php", "/home/user/.ssh/id_rsa", "/etc/shadow", "/tmp/malware.sh"]

def generate_event():
    event_type_choice = random.choice(ACTION_TYPES)
    severity = random.choice(SEVERITY_LEVELS)
    
    # Weight severity - mostly INFO/LOW
    if random.random() > 0.8:
        severity = random.choice(["HIGH", "CRITICAL"])
    elif random.random() > 0.5:
        severity = "MEDIUM"
    else:
        severity = random.choice(["INFO", "LOW"])

    timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat()
    
    event_data = {}
    
    if event_type_choice == "file_integrity":
        path = random.choice(FILES)
        
        # 10% chance to write a real EICAR file to trigger YARA
        if random.random() < 0.1:
            try:
                watch_path = os.environ.get("GUARDIAN_WATCH_PATH", f"{os.environ['HOME']}/projects/Guardian")
                eicar_path = os.path.join(watch_path, "eicar_test.txt")
                
                # Write EICAR string
                with open(eicar_path, "w") as f:
                    f.write("X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*")
                    
                path = eicar_path
                event_data = {
                    "type": "file_integrity",
                    "path": path,
                    "operation": "create",
                    "hash": "sha256:3395856ce81f2b7382dee72602f798b642f14140a36bc06717975d55d7b538e9"
                }
                severity = "CRITICAL" # Override severity
            except Exception as e:
                pass
        
        if not event_data:  # If not EICAR
            event_data = {
                "type": "file_integrity",
                "path": path,
                "operation": random.choice(["create", "modify", "delete", "chmod"]),
                "hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            }
    elif event_type_choice == "process_monitor":
        event_data = {
            "type": "process_monitor",
            "pid": random.randint(1000, 65535),
            "name": random.choice(["nginx", "ssh", "bash", "python", "crypto_miner"]),
            "cpu_usage": round(random.uniform(0.1, 99.9), 2),
            "memory_usage": random.randint(100000, 160000000)
        }
    elif event_type_choice == "network_socket":
        event_data = {
            "type": "network_socket",
            "local_addr": f"192.168.1.{random.randint(2, 254)}:443",
            "remote_addr": f"10.0.0.{random.randint(2, 254)}:{random.randint(1024, 65535)}",
            "protocol": random.choice(["tcp", "udp"]),
            "state": random.choice(["ESTABLISHED", "LISTEN", "CLOSE_WAIT"])
        }
    elif event_type_choice == "system_log":
        event_data = {
            "type": "system_log",
            "source": random.choice(["auth", "kernel", "daemon"]),
            "level": severity,
            "message": "Simulated system log message for security analysis"
        }
        
    # Merge specific event fields into the main event dictionary
    # because LogEvent uses #[serde(flatten)] for event_type
    
    event = {
        "id": str(uuid.uuid4()),
        "timestamp": timestamp,
        "severity": severity,
        # Flattened event type fields go here directly
        **event_data,
        "hostname": random.choice(HOSTNAMES),
        "tags": [event_type_choice, "simulated"],
        "rule_triggered": severity in ["HIGH", "CRITICAL"],
        "rule_name": "Simulated Rule Trigger" if severity in ["HIGH", "CRITICAL"] else None
    }
    
    return json.dumps(event)

def main():
    print("🚀 Connecting to Guardian Bridge...", file=sys.stderr)
    
    process = subprocess.Popen(
        ["./target/release/guardian-bridge"], 
        stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL,
        stderr=sys.stderr,
        text=True
    )

    try:
        count = 0
        while count < 50:  # Generate 50 events
            event_json = generate_event()
            
            if process.stdin:
                try:
                    process.stdin.write(event_json + "\n")
                    process.stdin.flush()
                    print(f"Sent event {count+1}", file=sys.stderr)
                    count += 1
                except BrokenPipeError:
                    print("Error: Bridge process closed unexpectedly", file=sys.stderr)
                    break

            time.sleep(0.05) # Fast generation
            
    except KeyboardInterrupt:
        pass
    finally:
        if process.stdin:
            process.stdin.close()
        process.wait()
        print("\n✅ Events populated!", file=sys.stderr)

if __name__ == "__main__":
    main()
