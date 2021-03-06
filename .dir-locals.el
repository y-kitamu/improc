;;; Directory Local Variables
;;; For more information see (info "(emacs) Directory Variables")

((rust-mode . ((dap-debug-template-name . "Rust::GDB Run Configuration")
               (dap-debug-template . (:type "gdb"
                                      :request "launch"
                                      :name "GDB::Run"
                                      :gdbpath "rust-gdb"
                                      :target nil
                                      :cwd nil))
               (lsp-rust-analyzer-debug-lens-extra-dap-args . (:MIMode "gdb"
                                                               :miDebuggerPath "rust-gdb"
                                                               :stopAtEntry t
                                                               :externalConsole
                                                               :json-false)))))
