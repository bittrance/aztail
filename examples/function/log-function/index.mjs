export default async function (context, _myTimer) {
  context.log("General log message")
  context.log.error("Error log message; oh no, badness!")
  context.log.warn("Warning log message; be careful.")
  context.log.info("Informational log message")
  context.log.verbose("Chatty log message")
}
