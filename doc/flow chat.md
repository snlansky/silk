                                                                node0 (tx into chain)                                               node1                                
    ---------------------------------------------------------------------------------------------------------                   -----------              

                                                   // reply when reveive the tx from the remote                                
                        ---------------------------------------------------------------------------------                                   
                        |                                                                               ^
                        |  ----------------------------------------------                               ^                                    
                        |  |               3.1.reply                    ^                               ^                                    
                        |  |                                            |                               |                                    
    +-----+  0.send   +----------+  1.forward   +------+  2.2.send  +-----------+    3.2.send      +---------+   broadcast:tx   +---------+               
    | APP |  ------>  | JSON-RPC |  --------->  | auth |  ------->  | consensus |  ------------->  | network |  <------------>  | network | ...
    +-----+           +----------+              +------+            +-----------+                  +---------+                  +---------+               
       ^                | ^  ^                   | ^ ^                    ^                               ^                                    
       |  4.reply       | ^  |     2.1.reply     | ^ | // tx from remote  |            4.broadcast tx    |                                    
       ------------------ ^  --------------------- ^ ---------------------|--------------------------------                                   
                          ^                        ^                      |                                                                      
                          ^                        |             6.2.sync | 5.package      +-------+  6.1.process                                        
                          |                        |                      -------------->  | chain |  ---->                                            
                          |                        |                                       +-------+      |                                    
                          |                        |       6.3.sync tx hash                  |  |         |                                        
                          |                        -------------------------------------------  |         |                                        
                          |                                                                     |     +---------+                                 
                          -----------------------------------------------------------------------     | rocksDB |                                 
                                                    6.4.reply                                         +---------+                                 