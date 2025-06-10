# Interference generator and optimal path finder

The program is developed to create interferences on 2D grid and to find a path using [NEOS Server: XML-RPC API](https://neos-server.org/neos/xml-rpc.html). 

## Usage

- user sets starting and ending points, draws interferences;
- chooses a template and solver which must be send to NEOS Server;
- ampl code is generated from a template (using [tera crate](https://docs.rs/tera/1.20.0/tera/index.html));
- all data is sent to NEOS Server using [XML-RPC API](https://neos-server.org/neos/xml-rpc.html);
- the result from NEOS Server is parsed using [nom crate](https://docs.rs/nom/8.0.0/nom/index.html) and displayed (or error message pop-ups).

https://github.com/user-attachments/assets/a30942f0-0b3e-426e-86c6-092bc64614bc
