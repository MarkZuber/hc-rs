use std::io::prelude::*;
use std::net::TcpStream;
use std::{thread, time};

// Protocol details found here:
// https://github.com/nicko88/HTWebRemote/blob/master/HTWebRemote/Devices/Controllers/EpsonControl.cs
// https://github.com/nicko88/HTWebRemote/blob/master/IP%20Control%20Spec%20Documents/Epson_PJ.pdf

pub struct EpsonProjector {
    device_address: String,
}

impl EpsonProjector {
    const PORT: i32 = 3629;

    pub fn new(device_address: &str) -> EpsonProjector {
        EpsonProjector {
            device_address: device_address.to_string(),
        }
    }

    pub fn turn_on(&self) {
        self.run_command("PWR ON");
    }

    pub fn turn_off(&self) {
        self.run_command("PWR OFF");
    }

    fn get_url(&self) -> String {
        format!("{}:{}", self.device_address, Self::PORT)
    }

    fn run_command(&self, command: &str) {
        let init_bytes: Vec<u8> = vec![
            0x45, 0x53, 0x43, 0x2F, 0x56, 0x50, 0x2E, 0x6E, 0x65, 0x74, 0x10, 0x03, 0x00, 0x00,
            0x00, 0x00,
        ];

        let mut stream = TcpStream::connect(self.get_url()).unwrap();
        stream.write(&init_bytes).unwrap();

        thread::sleep(time::Duration::from_millis(100));
        let final_command = format!("{}\r", command).to_ascii_uppercase();

        stream.write(&final_command.as_bytes()).unwrap();
    }

    /*
        We don't use / need it with what I'm doing, but this is the code to
        query the projector.  Putting it here in case I want to resurrect this.

        private static string Query(string ipAddress, string cmd)
        {
            using var socket = new SocketConnection(
                ipAddress, EpsonNetworkPort, SocketType.Stream, ProtocolType.Tcp);
            string dataResponse = "Error getting value";

            if (socket.Connect())
            {
                socket.SendData(s_init);
                Thread.Sleep(100);

                socket.SendData(Encoding.ASCII.GetBytes($"{cmd}\r"));

                _ = socket.ReceiveData();
                dataResponse = socket.ReceiveData();
            }

            try
            {
                dataResponse = dataResponse.Substring(
                    dataResponse.LastIndexOf('=') + 1,
                    dataResponse.Length - dataResponse.LastIndexOf(':') + 2);
            }
            catch (Exception e)
            {
                Debug.WriteLine($"Error parsing data: {dataResponse}\n\n{e.Message}");
            }

            return dataResponse;
        }
    */
}
