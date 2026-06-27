use gethostname;

// je recupere le hostname de la machine
pub fn current_hostname()-> String{
    gethostname::gethostname().to_string_lossy().to_string()
}