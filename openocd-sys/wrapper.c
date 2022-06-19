#ifndef BINDGEN
#define openocd_main __openocd_main
#endif

#include <openocd.c>

const struct command_registration __CONSTIFY_MACRO_COMMAND_REGISTRATION_DONE =
  COMMAND_REGISTRATION_DONE;

struct command_context*
__undo_static_setup_command_handler(Jim_Interp* interp)
{
    return setup_command_handler(interp);
}

int
__undo_static_openocd_thread(int argc,
                             char* argv[],
                             struct command_context* cmd_ctx)
{
    return openocd_thread(argc, argv, cmd_ctx);
}

int
__undo_static_register_commands(struct command_context* cmd_ctx,
                                const char* cmd_prefix,
                                const struct command_registration* cmds)
{
    return register_commands(cmd_ctx, cmd_prefix, cmds);
}
