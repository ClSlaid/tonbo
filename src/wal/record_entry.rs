use fusio::{Read, Write};

use crate::{
    record::{Key, Record},
    serdes::{Decode, Encode},
    timestamp::Timestamped,
};

pub(crate) enum RecordEntry<'r, R>
where
    R: Record,
{
    Encode((Timestamped<<R::Key as Key>::Ref<'r>>, Option<R::Ref<'r>>)),
    Decode((Timestamped<R::Key>, Option<R>)),
}

impl<R> Encode for RecordEntry<'_, R>
where
    R: Record,
{
    type Error = fusio::Error;

    async fn encode<W>(&self, writer: &mut W) -> Result<(), Self::Error>
    where
        W: Write + Unpin + Send,
    {
        if let RecordEntry::Encode((key, recode_ref)) = self {
            key.encode(writer).await.unwrap();
            recode_ref.encode(writer).await.unwrap();

            return Ok(());
        }
        unreachable!()
    }

    fn size(&self) -> usize {
        if let RecordEntry::Encode((key, recode_ref)) = self {
            return key.size() + recode_ref.size();
        }
        unreachable!()
    }
}

impl<Re> Decode for RecordEntry<'_, Re>
where
    Re: Record,
{
    type Error = fusio::Error;

    async fn decode<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: Read + Unpin,
    {
        let key = Timestamped::<Re::Key>::decode(reader).await.unwrap();
        let record = Option::<Re>::decode(reader).await.unwrap();

        Ok(RecordEntry::Decode((key, record)))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use fusio::Seek;

    use crate::{
        serdes::{Decode, Encode},
        timestamp::Timestamped,
        wal::record_entry::RecordEntry,
    };

    #[tokio::test]
    async fn encode_and_decode() {
        let entry: RecordEntry<'static, String> =
            RecordEntry::Encode((Timestamped::new("hello", 0.into()), Some("hello")));
        let mut bytes = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        entry.encode(&mut cursor).await.unwrap();

        let decode_entry = {
            cursor.seek(0).await.unwrap();

            RecordEntry::<'static, String>::decode(&mut cursor)
                .await
                .unwrap()
        };

        if let (RecordEntry::Encode((key_1, value_1)), RecordEntry::Decode((key_2, value_2))) =
            (entry, decode_entry)
        {
            assert_eq!(key_1.value, key_2.value.as_str());
            assert_eq!(key_1.ts, key_2.ts);
            assert_eq!(value_1, value_2.as_deref());

            return;
        }
        unreachable!()
    }
}
