// import { ReactNode } from 'react';
import { makeStyles } from "@material-ui/core/styles";
// import Paper from '@material-ui/core/Paper';

export const useEditorStyles = makeStyles((theme) => ({
  statusLineWrapper: {
    marginTop: theme.spacing(0.5),
    marginBottom: theme.spacing(-1),
  },
}));

// export function EditorWrapper({ children }: { children: ReactNode }) {
//   const classes = useStyles();

//   return (
//     <Paper elevation={1} className={classes.root}>
//       {children}
//     </Paper>
//   );
// }

// export function EditorStatusLineWrapper({ children }: { children: ReactNode }) {
//   const classes =
// }
