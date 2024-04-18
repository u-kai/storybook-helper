use std::path::Path;

use crate::component::Component;

#[derive(Debug, PartialEq, Clone)]
enum TSXToken {
    Export,
    Default,
    Const,
    Type,
    Equal,
    OpenCurly,
    CloseCurly,
    OpenParentis,
    CloseParentis,
    Colon,
    Comma,
    SemiColon,
    Var(String),
    Component(String),
}

/*

import { Alert, AlertTitle } from "@mui/material";
import React, { FC, useContext, useEffect } from "react";
import { AppErrorContext } from "../../contexts/error";

type Props = {
  timeOut: number;
  errorMessage?: string;
};

// propsは何かしらの明示された型か、リテラルなタイプ、もしくは空か
// 何かしらの明示された型であればその型のプロパティを再起的に拾ってくる必要がある
// そのファイルのコンポーネントはexport defaultかexport constのどちらかで、export constである場合は、関数である可能性もあるので、それがコンポーネントであるのかどうかの判定は必要

export const ErrorAlert: FC<Props> = (props: Props) => {
  const { appError, setAppError } = useContext(AppErrorContext);
  useEffect(() => {
    if (appError !== undefined) {
      setTimeout(() => {
        setAppError(undefined);
      }, props.timeOut);
    }
  }, [appError, setAppError]);

  return (
    <>
      {appError !== undefined || props.errorMessage ? (
        <Alert severity="error">
          <AlertTitle>Error</AlertTitle>
          {props.errorMessage ?? appError.message}
        </Alert>
      ) : null}
    </>
  );
};
*/

struct TSXContent(String);

impl TSXContent {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self(content))
    }
    fn to_component(self) -> Result<Component, String> {
        todo!()
    }
}
